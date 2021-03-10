use xous::{Message, ScalarMessage};
use xous::String;

/////////////////////// UART TYPE
#[derive(Debug)]
pub enum UartType {
    Kernel,
    Log,
    Application,
    Invalid,
}
// from/to for Xous messages
impl From<usize> for UartType {
    fn from(code: usize) -> Self {
        match code {
            0 => UartType::Kernel,
            1 => UartType::Log,
            2 => UartType::Application,
            _ => UartType::Invalid,
        }
    }
}
impl Into<usize> for UartType {
    fn into(self) -> usize {
        match self {
            UartType::Kernel => 0,
            UartType::Log => 1,
            UartType::Application => 2,
            UartType::Invalid => 3,
        }
    }
}
// for the actual bitmask going to hardware
impl Into<u32> for UartType {
    fn into(self) -> u32 {
        match self {
            UartType::Kernel => 0,
            UartType::Log => 1,
            UartType::Application => 2,
            UartType::Invalid => 3,
        }
    }
}

/////////////////////// I2C
use core::slice;
use core::ops::{Deref, DerefMut};
#[derive(Debug, Copy, Clone)]
pub struct I2cTransaction {
    bus_addr: u8,
    // write address and read address are encoded in the packet field below
    packet: [u8; 258], // up to 258 bytes total packet length; note cost is "same" b/c these are sent via 4kiB page remaps
    length: u8,
    // read or write type is encoded in the opcode
}
impl I2cTransaction {
    pub fn new() -> Self {
        I2cTransaction{ bus_addr: 0, packet: [0; 258], length: 0 }
    }
}
impl Deref for I2cTransaction {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(self as *const I2cTransaction as *const u8, core::mem::size_of::<I2cTransaction>())
               as &[u8]
        }
    }
}
impl DerefMut for I2cTransaction {
    fn deref_mut(&mut self) -> &mut[u8] {
        unsafe {
            slice::from_raw_parts_mut(self as *mut I2cTransaction as *mut u8, core::mem::size_of::<I2cTransaction>())
               as &mut [u8]
        }
    }
}
pub struct ArchivedI2cTransaction {
    ptr: rkyv::RelPtr,
}
pub struct I2cTransactionResolver {
    bytes_pos: usize,
}
impl rkyv::Resolve<I2cTransaction> for I2cTransactionResolver {
    type Archived = ArchivedI2cTransaction;
    fn resolve(self, pos: usize, _value: &I2cTransaction) -> Self::Archived {
        Self::Archived {
            ptr: unsafe {
                rkyv::RelPtr::new(pos + rkyv::offset_of!(ArchivedI2cTransaction, ptr), self.bytes_pos)
            },
        }
    }
}

impl rkyv::Archive for I2cTransaction {
    type Archived = ArchivedI2cTransaction;
    type Resolver = I2cTransactionResolver;

    fn archive<W: rkyv::Write + ?Sized>(&self, writer: &mut W) -> core::result::Result<Self::Resolver, W::Error> {
        let bytes_pos = writer.pos();
        writer.write(self.deref())?;
        Ok(Self::Resolver { bytes_pos })
    }
}
impl rkyv::Unarchive<I2cTransaction> for ArchivedI2cTransaction {
    fn unarchive(&self) -> I2cTransaction {
        let mut i2c: I2cTransaction = I2cTransaction::new();
        unsafe {
            let p = self.ptr.as_ptr() as *const u8;
            for(i, val) in i2c.deref_mut().iter_mut().enumerate() {
                *val = p.add(i).read();
            }
        };
        i2c
    }
}

////////////////////////////////// VIBE
#[derive(Debug)]
pub enum VibePattern {
    Short,
    Long,
    Double,
}
impl From<usize> for VibePattern {
    fn from(pattern: usize) -> Self {
        match pattern {
            0 => VibePattern::Long,
            1 => VibePattern::Double,
            _ => VibePattern::Short,
        }
    }
}
impl Into<usize> for VibePattern {
    fn into(self) -> usize {
        match self {
            VibePattern::Long => 0,
            VibePattern::Double => 1,
            VibePattern::Short => 0xffff_ffff,
        }
    }
}

//////////////////////////////// CLOCK GATING (placeholder)
#[derive(Debug)]
pub enum ClockMode {
    Low,
    AllOn,
}
impl From<usize> for ClockMode {
    fn from(mode: usize) -> Self {
        match mode {
            0 => ClockMode::Low,
            _ => ClockMode::AllOn,
        }
    }
}
impl Into<usize> for ClockMode {
    fn into(self) -> usize {
        match self {
            ClockMode::Low => 0,
            ClockMode::AllOn => 0xffff_ffff,
        }
    }
}

//////////////////////////////////// OPCODES
#[allow(dead_code)]
#[derive(Debug)]
pub enum Opcode {
    /// not tested - reboot
    RebootRequest,
    RebootSocConfirm, // all peripherals + CPU
    RebootCpuConfirm, // just the CPU, peripherals (in particular the USB debug bridge) keep state

    /// not tested - reboot address
    RebootVector(u32),

    /// not tested - set CRG parameters
    CrgMode(ClockMode),

    /// not tested -- set GPIO
    GpioDataOut(u32),
    GpioDataIn,
    GpioDataDrive(u32),
    GpioIntMask(u32),
    GpioIntAsFalling(u32),
    GpioIntPending,
    GpioIntEna(u32),
    GpioIntSubscribe(String<64>), // TODO

    /// not tested - set UART mux
    UartMux(UartType),

    /// not tested - information about the SoC build and revision
    InfoLitexId(String<64>), // TODO: returns the ASCII string baked into the FPGA that describes the FPGA build, inside Registration
    InfoDna,
    InfoGit,
    InfoPlatform,
    InfoTarget,
    InfoSeed,

    /// not tested -- power
    PowerAudio(bool),
    PowerSelf(bool), // setting this to false allows the EC to turn off our power
    PowerBoostMode(bool),
    EcSnoopAllow(bool),
    EcReset,
    EcPowerOn,
    SelfDestruct(u32), // requires a series of writes to enable

    /// not tested -- vibe
    Vibe(VibePattern),

    /// not tested -- xadc
    AdcVbus,
    AdcVccInt,
    AdcVccAux,
    AdcVccBram,
    AdcUsbN,
    AdcUsbP,
    AdcTemperature,
    AdcGpio5,
    AdcGpio2,

    /// not tested - I2C functions
    I2cWrite(I2cTransaction), /// LEFT OFF HERE -- need rkyv sending of messages
    I2cReadBlocking(I2cTransaction),
    I2cReadSubscribe(String<64>),
    I2cReadNonBlocking(I2cTransaction),

    /// not tested -- events
    EventComSubscribe(String<64>),
    EventRtcSubscribe(String<64>),
    EventUsbAttachSubscribe(String<64>),
    EventComEnable(bool),
    EventRtcEnable(bool),
    EventUsbAttachEnable(bool),
}

impl core::convert::TryFrom<& Message> for Opcode {
    type Error = &'static str;
    fn try_from(message: & Message) -> Result<Self, Self::Error> {
        match message {
            Message::Scalar(m) => match m.id {
                0 => Ok(Opcode::RebootRequest),
                1 => Ok(Opcode::RebootSocConfirm),
                21 => Ok(Opcode::RebootCpuConfirm),
                2 => Ok(Opcode::RebootVector(m.arg1 as u32)),
                3 => Ok(Opcode::CrgMode(m.arg1.into())),
                4 => Ok(Opcode::GpioDataOut(m.arg1 as u32)),
                5 => Ok(Opcode::GpioDataDrive(m.arg1 as u32)),
                6 => Ok(Opcode::GpioIntMask(m.arg1 as u32)),
                7 => Ok(Opcode::GpioIntAsFalling(m.arg1 as u32)),
                8 => Ok(Opcode::GpioIntEna(m.arg1 as u32)),
                9 => Ok(Opcode::UartMux(m.arg1.into())),
                10 => if m.arg1 == 0 {
                    Ok(Opcode::EventComEnable(false))
                } else {
                    Ok(Opcode::EventComEnable(true))
                },
                11 => if m.arg1 == 0 {
                    Ok(Opcode::EventRtcEnable(false))
                } else {
                    Ok(Opcode::EventRtcEnable(true))
                },
                12 => if m.arg1 == 0 {
                    Ok(Opcode::EventUsbAttachEnable(false))
                } else {
                    Ok(Opcode::EventUsbAttachEnable(true))
                },
                13 => if m.arg1 == 0 {
                    Ok(Opcode::PowerAudio(false))
                } else {
                    Ok(Opcode::PowerAudio(true))
                },
                14 => if m.arg1 == 0 {
                    Ok(Opcode::PowerSelf(false))
                } else {
                    Ok(Opcode::PowerSelf(true))
                },
                15 => if m.arg1 == 0 {
                    Ok(Opcode::PowerBoostMode(false))
                } else {
                    Ok(Opcode::PowerBoostMode(true))
                },
                16 => if m.arg1 == 0 {
                    Ok(Opcode::EcSnoopAllow(false))
                } else {
                    Ok(Opcode::EcSnoopAllow(true))
                },
                17 => Ok(Opcode::EcReset),
                18 => Ok(Opcode::EcPowerOn),
                19 => Ok(Opcode::SelfDestruct(m.arg1 as u32)),
                20 => Ok(Opcode::Vibe(m.arg1.into())),
                // note 21 is used for RebootCpuConfirm
                _ => Err("LLIO api: unknown Scalar ID"),
            },
            Message::BlockingScalar(m) => match m.id {
                0x101 => Ok(Opcode::GpioDataIn),
                0x102 => Ok(Opcode::GpioIntPending),
                0x104 => Ok(Opcode::InfoDna),
                0x105 => Ok(Opcode::InfoGit),
                0x106 => Ok(Opcode::InfoPlatform),
                0x107 => Ok(Opcode::InfoTarget),
                0x108 => Ok(Opcode::InfoSeed),
                0x109 => Ok(Opcode::AdcVbus),
                0x10A => Ok(Opcode::AdcVccInt),
                0x10B => Ok(Opcode::AdcVccAux),
                0x10C => Ok(Opcode::AdcVccBram),
                0x10D => Ok(Opcode::AdcUsbN),
                0x10E => Ok(Opcode::AdcUsbP),
                0x10F => Ok(Opcode::AdcTemperature),
                0x110 => Ok(Opcode::AdcGpio5),
                0x111 => Ok(Opcode::AdcGpio2),
                _ => Err("LLIO api: unknown BlockingScalar ID"),
            },
            _ => Err("unhandled message type"),
        }
    }
}

impl Into<Message> for Opcode {
    fn into(self) -> Message {
        match self {
            // scalars
            Opcode::RebootRequest => Message::Scalar(ScalarMessage {
                id: 0,
                arg1: 0, arg2: 0, arg3: 0, arg4: 0,
            }),
            Opcode::RebootSocConfirm => Message::Scalar(ScalarMessage {
                id: 1,
                arg1: 0, arg2: 0, arg3: 0, arg4: 0,
            }),
            Opcode::RebootCpuConfirm => Message::Scalar(ScalarMessage {
                id: 21,
                arg1: 0, arg2: 0, arg3: 0, arg4: 0,
            }),
            Opcode::RebootVector(vector) => Message::Scalar(ScalarMessage {
                id: 2,
                arg1: vector as usize, arg2: 0, arg3: 0, arg4: 0,
            }),
            Opcode::CrgMode(mode) => Message::Scalar(ScalarMessage {
                id: 3,
                arg1: mode as usize, arg2: 0, arg3: 0, arg4: 0,
            }),
            Opcode::GpioDataOut(data) => Message::Scalar(ScalarMessage {
                id: 4,
                arg1: data as usize, arg2: 0, arg3: 0, arg4: 0,
            }),
            Opcode::GpioDataDrive(drive_bits) => Message::Scalar(ScalarMessage {
                id: 5,
                arg1: drive_bits as usize, arg2: 0, arg3: 0, arg4: 0,
            }),
            Opcode::GpioIntMask(mask) => Message::Scalar(ScalarMessage {
                id: 6,
                arg1: mask as usize, arg2: 0, arg3: 0, arg4: 0,
            }),
            Opcode::GpioIntAsFalling(falling) => Message::Scalar(ScalarMessage {
                id: 7,
                arg1: falling as usize, arg2: 0, arg3: 0, arg4: 0,
            }),
            Opcode::GpioIntEna(enable) => Message::Scalar(ScalarMessage {
                id: 8,
                arg1: enable as usize, arg2: 0, arg3: 0, arg4: 0,
            }),
            Opcode::UartMux(uarttype) => Message::Scalar(ScalarMessage {
                id: 9,
                arg1: uarttype as usize, arg2: 0, arg3: 0, arg4: 0,
            }),
            Opcode::EventComEnable(ena) => if ena {
                Message::Scalar(ScalarMessage {
                    id: 10,
                    arg1: 1, arg2: 0, arg3: 0, arg4: 0,
                })
            } else {
                Message::Scalar(ScalarMessage {
                    id: 10,
                    arg1: 0, arg2: 0, arg3: 0, arg4: 0,
                })
            },
            Opcode::EventRtcEnable(ena) => if ena {
                Message::Scalar(ScalarMessage {
                    id: 11,
                    arg1: 1, arg2: 0, arg3: 0, arg4: 0,
                })
            } else {
                Message::Scalar(ScalarMessage {
                    id: 11,
                    arg1: 0, arg2: 0, arg3: 0, arg4: 0,
                })
            },
            Opcode::EventUsbAttachEnable(ena) => if ena {
                Message::Scalar(ScalarMessage {
                    id: 12,
                    arg1: 1, arg2: 0, arg3: 0, arg4: 0,
                })
            } else {
                Message::Scalar(ScalarMessage {
                    id: 12,
                    arg1: 0, arg2: 0, arg3: 0, arg4: 0,
                })
            },
            Opcode::PowerAudio(ena) => if ena {
                Message::Scalar(ScalarMessage {
                    id: 13,
                    arg1: 1, arg2: 0, arg3: 0, arg4: 0,
                })
            } else {
                Message::Scalar(ScalarMessage {
                    id: 13,
                    arg1: 0, arg2: 0, arg3: 0, arg4: 0,
                })
            },
            Opcode::PowerSelf(ena) => if ena {
                Message::Scalar(ScalarMessage {
                    id: 14,
                    arg1: 1, arg2: 0, arg3: 0, arg4: 0,
                })
            } else {
                Message::Scalar(ScalarMessage {
                    id: 14,
                    arg1: 0, arg2: 0, arg3: 0, arg4: 0,
                })
            },
            Opcode::PowerBoostMode(ena) => if ena {
                Message::Scalar(ScalarMessage {
                    id: 15,
                    arg1: 1, arg2: 0, arg3: 0, arg4: 0,
                })
            } else {
                Message::Scalar(ScalarMessage {
                    id: 15,
                    arg1: 0, arg2: 0, arg3: 0, arg4: 0,
                })
            },
            Opcode::EcSnoopAllow(ena) => if ena {
                Message::Scalar(ScalarMessage {
                    id: 16,
                    arg1: 1, arg2: 0, arg3: 0, arg4: 0,
                })
            } else {
                Message::Scalar(ScalarMessage {
                    id: 16,
                    arg1: 0, arg2: 0, arg3: 0, arg4: 0,
                })
            },
            Opcode::EcReset => Message::Scalar(ScalarMessage {
                id: 17,
                arg1: 0, arg2: 0, arg3: 0, arg4: 0,
            }),
            Opcode::EcPowerOn => Message::Scalar(ScalarMessage {
                id: 18,
                arg1: 0, arg2: 0, arg3: 0, arg4: 0,
            }),
            Opcode::SelfDestruct(code) => Message::Scalar(ScalarMessage {
                id: 19,
                arg1: code as usize, arg2: 0, arg3: 0, arg4: 0,
            }),
            Opcode::Vibe(pattern) => Message::Scalar(ScalarMessage {
                id: 20,
                arg1: pattern as usize, arg2: 0, arg3: 0, arg4: 0,
            }),
            // note 21 is used by RebootCpuConfirm

            // blocking scalars
            Opcode::GpioDataIn => Message::BlockingScalar(ScalarMessage {
                id: 0x101,
                arg1: 0, arg2: 0, arg3: 0, arg4: 0,
            }),
            Opcode::GpioIntPending => Message::BlockingScalar(ScalarMessage {
                id: 0x102,
                arg1: 0, arg2: 0, arg3: 0, arg4: 0,
            }),
            Opcode::InfoDna => Message::BlockingScalar(ScalarMessage {
                id: 0x104,
                arg1: 0, arg2: 0, arg3: 0, arg4: 0,
            }),
            Opcode::InfoGit => Message::BlockingScalar(ScalarMessage {
                id: 0x105,
                arg1: 0, arg2: 0, arg3: 0, arg4: 0,
            }),
            Opcode::InfoPlatform => Message::BlockingScalar(ScalarMessage {
                id: 0x106,
                arg1: 0, arg2: 0, arg3: 0, arg4: 0,
            }),
            Opcode::InfoTarget => Message::BlockingScalar(ScalarMessage {
                id: 0x107,
                arg1: 0, arg2: 0, arg3: 0, arg4: 0,
            }),
            Opcode::InfoSeed => Message::BlockingScalar(ScalarMessage {
                id: 0x108,
                arg1: 0, arg2: 0, arg3: 0, arg4: 0,
            }),
            Opcode::AdcVbus => Message::BlockingScalar(ScalarMessage {
                id: 0x109,
                arg1: 0, arg2: 0, arg3: 0, arg4: 0,
            }),
            Opcode::AdcVccInt => Message::BlockingScalar(ScalarMessage {
                id: 0x10A,
                arg1: 0, arg2: 0, arg3: 0, arg4: 0,
            }),
            Opcode::AdcVccAux => Message::BlockingScalar(ScalarMessage {
                id: 0x10B,
                arg1: 0, arg2: 0, arg3: 0, arg4: 0,
            }),
            Opcode::AdcVccBram => Message::BlockingScalar(ScalarMessage {
                id: 0x10C,
                arg1: 0, arg2: 0, arg3: 0, arg4: 0,
            }),
            Opcode::AdcUsbN => Message::BlockingScalar(ScalarMessage {
                id: 0x10D,
                arg1: 0, arg2: 0, arg3: 0, arg4: 0,
            }),
            Opcode::AdcUsbP => Message::BlockingScalar(ScalarMessage {
                id: 0x10E,
                arg1: 0, arg2: 0, arg3: 0, arg4: 0,
            }),
            Opcode::AdcTemperature => Message::BlockingScalar(ScalarMessage {
                id: 0x10F,
                arg1: 0, arg2: 0, arg3: 0, arg4: 0,
            }),
            Opcode::AdcGpio5 => Message::BlockingScalar(ScalarMessage {
                id: 0x110,
                arg1: 0, arg2: 0, arg3: 0, arg4: 0,
            }),
            Opcode::AdcGpio2 => Message::BlockingScalar(ScalarMessage {
                id: 0x110,
                arg1: 0, arg2: 0, arg3: 0, arg4: 0,
            }),
            _ => panic!("opcode not handled -- maybe you meant to use one of the direct APIs")
        }
    }
}