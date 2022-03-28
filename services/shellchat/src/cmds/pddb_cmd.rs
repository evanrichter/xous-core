use crate::{ShellCmdApi, CommonEnv};
use xous_ipc::String;

pub struct PddbCmd {
    pddb: pddb::Pddb,
}
impl PddbCmd {
    pub fn new(_xns: &xous_names::XousNames) -> PddbCmd {
        PddbCmd {
            pddb: pddb::Pddb::new(),
        }
    }
}

impl<'a> ShellCmdApi<'a> for PddbCmd {
    cmd_api!(pddb); // inserts boilerplate for command API

    fn process(&mut self, args: String::<1024>, _env: &mut CommonEnv) -> Result<Option<String::<1024>>, xous::Error> {
        use core::fmt::Write;
        let mut ret = String::<1024>::new();
        #[cfg(not(feature="pddbtest"))]
        let helpstring = "pddb [basislist] [dictlist] [keylist] [query] [dictdelete] [keydelete]";
        #[cfg(feature="pddbtest")]
        let helpstring = "pddb [basislist] [dictlist] [keylist] [query] [dictdelete] [keydelete] [test]";

        let mut tokens = args.as_str().unwrap().split(' ');
        if let Some(sub_cmd) = tokens.next() {
            match sub_cmd {
                "basislist" => {
                    let bases = self.pddb.list_basis();
                    for basis in bases {
                        write!(ret, "{}\n", basis).unwrap();
                    }
                    /* // example of using .get with a callback
                    self.pddb.get("foo", "bar", None, false, false,
                        Some({
                            let cid = cid.clone();
                            let counter = self.counter.clone();
                            move || {
                            xous::send_message(cid, xous::Message::new_scalar(0, counter as usize, 0, 0, 0)).expect("couldn't send");
                        }})
                    ).unwrap();*/
                }
                "query" => {
                    if let Some(descriptor) = tokens.next() {
                        if let Some((dict, keyname)) = descriptor.split_once(':') {
                            match self.pddb.get(dict, keyname, None,
                                false, false, None, None::<fn()>) {
                                Ok(mut key) => {
                                    use std::io::Read;
                                    let mut readbuf = [0u8; 512]; // up to the first 512 chars of the key
                                    match key.read(&mut readbuf) {
                                        Ok(len) => {
                                            match std::string::String::from_utf8(readbuf[..len].to_vec()) {
                                                Ok(s) => {
                                                    write!(ret, "{}", s).unwrap();
                                                }
                                                _ => {
                                                    for &b in readbuf[..len].iter() {
                                                        match write!(ret, "{:02x} ", b) {
                                                            Ok(_) => (),
                                                            Err(_) => break, // we can overflow our return buffer returning hex chars
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        _ => write!(ret, "Error encountered reading {}:{}", dict, keyname).unwrap()
                                    }
                                }
                                _ => write!(ret, "{}:{} not found or other error", dict, keyname).unwrap()
                            }
                        } else {
                            write!(ret, "Query is of form 'dict:key'").unwrap();
                        }
                    } else {
                        write!(ret, "Missing query of form 'dict:key'").unwrap();
                    }
                }
                "keydelete" => {
                    if let Some(descriptor) = tokens.next() {
                        if let Some((dict, keyname)) = descriptor.split_once(':') {
                            match self.pddb.delete_key(dict, keyname, None) {
                                Ok(_) => {
                                    write!(ret, "Deleted {}:{}\n", dict, keyname).unwrap();
                                    // you must call sync after all deletions are done
                                    write!(ret, "Sync: {}",
                                        self.pddb.sync()
                                        .map_or_else(|e| e.to_string(), |_| "Ok".to_string())
                                    ).unwrap();
                                }
                                Err(e) => write!(ret, "{}:{} not found or other error: {:?}", dict, keyname, e).unwrap(),
                            }
                        } else {
                            write!(ret, "Specify key with form 'dict:key'").unwrap();
                        }
                    } else {
                        write!(ret, "Missing spec of form 'dict:key'").unwrap();
                    }
                }
                "dictdelete" => {
                    if let Some(dict) = tokens.next() {
                        match self.pddb.delete_dict(dict, None) {
                            Ok(_) => {
                                write!(ret, "Deleted dictionary {}\n", dict).unwrap();
                                // you must call sync after all deletions are done
                                write!(ret, "Sync: {}",
                                    self.pddb.sync()
                                    .map_or_else(|e| e.to_string(), |_| "Ok".to_string())
                                ).unwrap();
                            }
                            Err(e) => write!(ret, "{} not found or other error: {:?}", dict, e).unwrap()
                        }
                    } else {
                        write!(ret, "Missing dictionary name").unwrap();
                    }
                }
                "keylist" => {
                    if let Some(dict) = tokens.next() {
                        match self.pddb.list_keys(dict, None) {
                            Ok(list) => {
                                let checked_len = if list.len() > 6 {
                                    write!(ret, "First 6 keys of {}:", list.len()).unwrap();
                                    6
                                } else {
                                    list.len()
                                };
                                for i in 0..checked_len {
                                    let sep = if i != checked_len - 1 {
                                        ", "
                                    } else {
                                        ""
                                    };
                                    match write!(ret, "{}{}", list[i], sep) {
                                        Ok(_) => (),
                                        Err(_) => break, // overflowed return buffer
                                    }
                                }
                            }
                            Err(_) => write!(ret, "{} does not exist or other error", dict).ok().unwrap_or(()),
                        }
                    } else {
                        write!(ret, "Missing dictionary name").unwrap();
                    }
                }
                "dictlist" => {
                    match self.pddb.list_dict(None) {
                        Ok(list) => {
                            let checked_len = if list.len() > 6 {
                                write!(ret, "First 6 dicts of {}:", list.len()).unwrap();
                                6
                            } else {
                                list.len()
                            };
                            for i in 0..checked_len {
                                let sep = if i != checked_len - 1 {
                                    ", "
                                } else {
                                    ""
                                };
                                match write!(ret, "{}{}", list[i], sep) {
                                    Ok(_) => (),
                                    Err(_) => break, // overflowed return buffer
                                }
                            }
                        }
                        Err(_) => write!(ret, "Error encountered listing dictionaries").ok().unwrap_or(()),
                    }
                }
                // note that this feature only works in hosted mode
                #[cfg(feature="pddbtest")]
                "test" => {
                    // zero-length key test
                    let mut test_handle = pddb::Pddb::new();
                    // build a key, but don't write to it.
                    let _ = test_handle.get(
                        "test",
                        "zerolength",
                        None, true, true,
                        Some(8),
                        None::<fn()>
                    ).expect("couldn't build empty key");
                    self.pddb.sync().unwrap();
                    self.pddb.dbg_remount().unwrap();
                    self.pddb.dbg_dump("std_test1").unwrap();
                    write!(ret, "dumped std_test1").unwrap();
                }
                _ => {
                    write!(ret, "{}", helpstring).unwrap();
                }
            }

        } else {
            write!(ret, "{}", helpstring).unwrap();
        }
        Ok(Some(ret))
    }
}
