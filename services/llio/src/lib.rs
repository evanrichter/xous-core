#![cfg_attr(target_os = "none", no_std)]

/// This is the API that other servers use to call the COM. Read this code as if you
/// are calling these functions inside a different process.
pub mod api;

use xous::{send_message, CID};

pub fn allow_power_off(cid: CID, allow: bool) -> Result<(), xous::Error> {
    send_message(cid, api::Opcode::PowerSelf(!allow).into()).map(|_| ())
}

pub fn allow_ec_snoop(cid: CID, allow: bool) -> Result<(), xous::Error> {
    send_message(cid, api::Opcode::EcSnoopAllow(allow).into()).map(|_| ())
}

pub fn adc_vbus(cid: CID) -> Result<u16, xous::Error> {
    let response = send_message(cid, api::Opcode::AdcVbus.into())?;
    if let xous::Result::Scalar1(val) = response {
        Ok(val as u16)
    } else {
        log::error!("LLIO: unexpected return value: {:#?}", response);
        Err(xous::Error::InternalError)
    }
}
pub fn adc_vccint(cid: CID) -> Result<u16, xous::Error> {
    let response = send_message(cid, api::Opcode::AdcVccInt.into())?;
    if let xous::Result::Scalar1(val) = response {
        Ok(val as u16)
    } else {
        log::error!("LLIO: unexpected return value: {:#?}", response);
        Err(xous::Error::InternalError)
    }
}
pub fn adc_vccaux(cid: CID) -> Result<u16, xous::Error> {
    let response = send_message(cid, api::Opcode::AdcVccAux.into())?;
    if let xous::Result::Scalar1(val) = response {
        Ok(val as u16)
    } else {
        log::error!("LLIO: unexpected return value: {:#?}", response);
        Err(xous::Error::InternalError)
    }
}
pub fn adc_vccbram(cid: CID) -> Result<u16, xous::Error> {
    let response = send_message(cid, api::Opcode::AdcVccBram.into())?;
    if let xous::Result::Scalar1(val) = response {
        Ok(val as u16)
    } else {
        log::error!("LLIO: unexpected return value: {:#?}", response);
        Err(xous::Error::InternalError)
    }
}
pub fn adc_usb_n(cid: CID) -> Result<u16, xous::Error> {
    let response = send_message(cid, api::Opcode::AdcUsbN.into())?;
    if let xous::Result::Scalar1(val) = response {
        Ok(val as u16)
    } else {
        log::error!("LLIO: unexpected return value: {:#?}", response);
        Err(xous::Error::InternalError)
    }
}
pub fn adc_usb_p(cid: CID) -> Result<u16, xous::Error> {
    let response = send_message(cid, api::Opcode::AdcUsbP.into())?;
    if let xous::Result::Scalar1(val) = response {
        Ok(val as u16)
    } else {
        log::error!("LLIO: unexpected return value: {:#?}", response);
        Err(xous::Error::InternalError)
    }
}
pub fn adc_temperature(cid: CID) -> Result<u16, xous::Error> {
    let response = send_message(cid, api::Opcode::AdcTemperature.into())?;
    if let xous::Result::Scalar1(val) = response {
        Ok(val as u16)
    } else {
        log::error!("LLIO: unexpected return value: {:#?}", response);
        Err(xous::Error::InternalError)
    }
}
pub fn adc_gpio5(cid: CID) -> Result<u16, xous::Error> {
    let response = send_message(cid, api::Opcode::AdcGpio5.into())?;
    if let xous::Result::Scalar1(val) = response {
        Ok(val as u16)
    } else {
        log::error!("LLIO: unexpected return value: {:#?}", response);
        Err(xous::Error::InternalError)
    }
}
pub fn adc_gpio2(cid: CID) -> Result<u16, xous::Error> {
    let response = send_message(cid, api::Opcode::AdcGpio2.into())?;
    if let xous::Result::Scalar1(val) = response {
        Ok(val as u16)
    } else {
        log::error!("LLIO: unexpected return value: {:#?}", response);
        Err(xous::Error::InternalError)
    }
}