// SPDX-License-Identifier: GPL-2.0

//! Rust Realtek PHYs driver
use kernel::bindings;
use kernel::c_str;
use kernel::error;
use kernel::net::phy;
use kernel::prelude::*;

module! {
    type: RustRealtekPhy,
    name: "rust_realtek_phy",
    author: "Rust for Linux Contributors",
    description: "Rust Realtek PHYs driver",
    license: "GPL",
}

struct PhyFeGe {}

#[vtable]
impl phy::Driver for PhyFeGe {
    fn match_phy_device(dev: &mut phy::Device) -> bool {
        if dev.id() == RustRealtekPhy::RTL_GENERIC_PHYID
            && !RustRealtekPhy::is_supports_2_5gbps(dev).is_ok_and(|x| x)
        {
            true
        } else {
            false
        }
    }

    fn read_page(dev: &mut phy::Device) -> Result<i32> {
        dev.lockless_read(RustRealtekPhy::RTL821X_PAGE_SELECT)
    }

    fn write_page(dev: &mut phy::Device, page: i32) -> Result {
        dev.lockless_write(RustRealtekPhy::RTL821X_PAGE_SELECT, page as u16)
    }

    fn read_status(dev: &mut phy::Device) -> Result {
        dev.read_status()?;
        RustRealtekPhy::set_speed(dev)?;
        Ok(())
    }

    fn suspend(dev: &mut phy::Device) -> Result {
        dev.suspend()
    }

    fn resume(dev: &mut phy::Device) -> Result {
        dev.resume()
    }

    fn read_mmd(dev: &mut phy::Device, devnum: i32, regnum: u16) -> Result<i32> {
        if devnum as u32 == bindings::MDIO_MMD_PCS && regnum as u32 == bindings::MDIO_PCS_EEE_ABLE {
            PhyFeGe::write_page(dev, 0xa5c)?;
            let ret = dev.lockless_read(0x12)?;
            PhyFeGe::write_page(dev, 0)?;
            Ok(ret)
        } else if devnum as u32 == bindings::MDIO_MMD_AN
            && regnum as u32 == bindings::MDIO_AN_EEE_ADV
        {
            PhyFeGe::write_page(dev, 0xa5d)?;
            let ret = dev.lockless_read(0x10)?;
            PhyFeGe::write_page(dev, 0)?;
            Ok(ret)
        } else if devnum as u32 == bindings::MDIO_MMD_AN
            && regnum as u32 == bindings::MDIO_AN_EEE_LPABLE
        {
            PhyFeGe::write_page(dev, 0xa5d)?;
            let ret = dev.lockless_read(0x11)?;
            PhyFeGe::write_page(dev, 0)?;
            Ok(ret)
        } else {
            Err(error::code::ENOTSUPP)
        }
    }

    fn write_mmd(dev: &mut phy::Device, devnum: i32, regnum: u16, val: u16) -> Result {
        if devnum as u32 == bindings::MDIO_MMD_AN && regnum as u32 == bindings::MDIO_AN_EEE_ADV {
            PhyFeGe::write_page(dev, 0xa5d)?;
            dev.lockless_write(0x10, val)?;
            PhyFeGe::write_page(dev, 0)?;
            Ok(())
        } else {
            Err(error::code::ENOTSUPP)
        }
    }
}

struct RustRealtekPhy {
    _reg: phy::Registration<1>,
}

impl RustRealtekPhy {
    const RTL_GENERIC_PHYID: u32 = 0x001cc800;
    const RTL821X_PAGE_SELECT: u32 = 0x1f;
    const RTL_SUPPORTS_2500FULL: u32 = 1 << 13;
    const RTLGEN_SPEED_MASK: u32 = 0x0630;

    fn set_speed(dev: &mut phy::Device) -> Result {
        if !dev.link() {
            return Ok(());
        }

        let ret = dev.read_paged(0xa43, 0x12)?;
        match ret as u32 & RustRealtekPhy::RTLGEN_SPEED_MASK {
            0x0000 => dev.set_speed(10),
            0x0010 => dev.set_speed(100),
            0x0020 => dev.set_speed(1000),
            0x0200 => dev.set_speed(10000),
            0x0210 => dev.set_speed(2500),
            0x0220 => dev.set_speed(5000),
            _ => {}
        }
        Ok(())
    }

    fn is_supports_2_5gbps(dev: &mut phy::Device) -> Result<bool> {
        dev.write(RustRealtekPhy::RTL821X_PAGE_SELECT, 0xa61)?;
        let val = dev.read(0x13)?;
        dev.write(RustRealtekPhy::RTL821X_PAGE_SELECT, 0)?;
        if val > 0 && val as u32 & RustRealtekPhy::RTL_SUPPORTS_2500FULL > 0 {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl kernel::Module for RustRealtekPhy {
    fn init(module: &'static ThisModule) -> Result<Self> {
        pr_info!("Rust Realtek phy driver\n");

        let mut reg: phy::Registration<1> = phy::Registration::new(module);

        reg.register(&phy::Adapter::<PhyFeGe>::new(c_str!(
            "Generic FE-GE Realtek PHY"
        )))?;

        Ok(RustRealtekPhy { _reg: reg })
    }
}

impl Drop for RustRealtekPhy {
    fn drop(&mut self) {
        pr_info!("Rust Realtek phy driver (exit)\n");
    }
}

kernel::phy_module_device_table!(
    __mod_mdio__realtek_table_device_table,
    [(RustRealtekPhy::RTL_GENERIC_PHYID, 0xfffffc00)]
);
