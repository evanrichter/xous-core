use crate::{ShellCmdApi,CommonEnv};
use xous_ipc::String;

#[derive(Debug)]
pub struct Sleep {
    rtc: rtc::Rtc,
    susres: susres::Susres,
}
impl Sleep {
    pub fn new(xns: &xous_names::XousNames) -> Self {
        let rtc = rtc::Rtc::new(&xns).unwrap();
        Sleep {
            rtc,
            susres: susres::Susres::new_without_hook(&xns).unwrap(),
        }
    }
}

fn kill_thread(bounce: usize) {
    log::info!("Self destruct thread active.");

    let xns = xous_names::XousNames::new().unwrap();
    let llio = llio::Llio::new(&xns).unwrap();
    let com = com::Com::new(&xns).unwrap();
    let ticktimer = ticktimer_server::Ticktimer::new().unwrap();
    ticktimer.sleep_ms(3000).unwrap();

    loop {
        log::info!("Initiating self destruct sequence...");
        if bounce != 0 {
            // slip in a ship mode command. should take long enough to execute that the kill goes through
            com.ship_mode().unwrap();
            ticktimer.sleep_ms(100).unwrap();
        }
        llio.self_destruct(0x2718_2818).unwrap();
        llio.self_destruct(0x3141_5926).unwrap();
        com.power_off_soc().unwrap();
        ticktimer.sleep_ms(100).unwrap();
        log::info!("If you can read this, we failed to destroy ourselves!");
    }
}

fn susres_loop() {
    log::info!("suspend/resume stress test active");

    let xns = xous_names::XousNames::new().unwrap();
    let susres = susres::Susres::new_without_hook(&xns).unwrap();
    let rtc = rtc::Rtc::new(&xns).unwrap();
    let ticktimer = ticktimer_server::Ticktimer::new().unwrap();

    ticktimer.sleep_ms(1500).unwrap();

    let mut iters = 0;
    loop {
        log::info!("suspend/resume cycle: {}", iters);
        rtc.set_wakeup_alarm(4).unwrap();
        susres.initiate_suspend().unwrap();

        ticktimer.sleep_ms(8000).unwrap();
        iters += 1;
    }
}

impl<'a> ShellCmdApi<'a> for Sleep {
    cmd_api!(sleep); // inserts boilerplate for command API

    fn process(&mut self, args: String::<1024>, env: &mut CommonEnv) -> Result<Option<String::<1024>>, xous::Error> {
        use core::fmt::Write;

        let mut ret = String::<1024>::new();
        let helpstring = "sleep [now] [current] [ship] [kill] [coldboot] [killbounce] [sus] [stress] [crypton] [cryptoff] [wfioff] [wfion] [debugwfi]";

        let mut tokens = args.as_str().unwrap().split(' ');

        // in all cases, we want the boost to be off to ensure a clean shutdown
        env.com.set_boost(false).unwrap();
        env.llio.boost_on(false).unwrap();

        if let Some(sub_cmd) = tokens.next() {
            match sub_cmd {
                "crypton" => {
                    env.llio.crypto_on(true).unwrap();
                    write!(ret, "crypto power is now on").unwrap();
                }
                "cryptoff" => {
                    env.llio.crypto_on(false).unwrap();
                    write!(ret, "crypto power is now off").unwrap();
                }
                "sus" => {
                    self.susres.initiate_suspend().unwrap();
                    // the message below is sent after we wake up
                    write!(ret, "Resumed from sleep!").unwrap();
                }
                "stress" => {
                    xous::create_thread_0(susres_loop).unwrap();
                    write!(ret, "Starting suspend/resume stress test. Hard reboot required to exit.").unwrap();
                }
                "now" => {
                    if ((env.llio.adc_vbus().unwrap() as f64) * 0.005033) > 1.5 {
                        // if power is plugged in, deny powerdown request
                        write!(ret, "System can't sleep while charging. Unplug charging cable and try again.").unwrap();
                    } else {
                        if Ok(true) == env.gam.powerdown_request() {
                            env.ticktimer.sleep_ms(500).unwrap(); // let the screen redraw

                            // allow EC to snoop, so that it can wake up the system
                            env.llio.allow_ec_snoop(true).unwrap();
                            // allow the EC to power me down
                            env.llio.allow_power_off(true).unwrap();
                            // now send the power off command
                            env.com.power_off_soc().unwrap();

                            log::info!("CMD: powering down now!");
                            // pause execution, nothing after this should be reachable
                            env.ticktimer.sleep_ms(2000).unwrap(); // should power off within 2 seconds
                            log::info!("CMD: if you can read this, power down failed!");
                        }
                        write!(ret, "Powerdown request denied").unwrap();
                    }
                }
                "current" => {
                    if let Some(i) = env.com.get_standby_current().unwrap() {
                        write!(ret, "Last standby current was {}mA", i).unwrap();
                    } else {
                        write!(ret, "Standby current measurement not initialized.").unwrap();
                    }
                }
                "ship" => {
                    if ((env.llio.adc_vbus().unwrap() as f64) * 0.005033) > 1.5 {
                        // if power is plugged in, deny powerdown request
                        write!(ret, "System can't go into ship mode while charging. Unplug charging cable and try again.").unwrap();
                    } else {
                        if Ok(true) == env.gam.shipmode_blank_request() {
                            env.ticktimer.sleep_ms(500).unwrap(); // let the screen redraw

                            // allow EC to snoop, so that it can wake up the system
                            env.llio.allow_ec_snoop(true).unwrap();
                            // allow the EC to power me down
                            env.llio.allow_power_off(true).unwrap();
                            // now send the power off command
                            env.com.ship_mode().unwrap();

                            // now send the power off command
                            env.com.power_off_soc().unwrap();

                            log::info!("CMD: ship mode now!");
                            // pause execution, nothing after this should be reachable
                            env.ticktimer.sleep_ms(10000).unwrap(); // ship mode happens in 10 seconds
                            log::info!("CMD: if you can read this, ship mode failed!");
                        }
                        write!(ret, "Ship mode request denied").unwrap();
                    }
                }
                "coldboot" => {
                    if ((env.llio.adc_vbus().unwrap() as f64) * 0.005033) > 1.5 {
                        // if power is plugged in, deny powerdown request
                        write!(ret, "System can't cold boot while charging. Unplug charging cable and try again.").unwrap();
                    } else {
                        if Ok(true) == env.gam.powerdown_request() {
                            env.ticktimer.sleep_ms(500).unwrap(); // let the screen redraw

                            // set a wakeup alarm a couple seconds from now -- this is the coldboot
                            self.rtc.set_wakeup_alarm(4).unwrap();

                            // allow EC to snoop, so that it can wake up the system
                            env.llio.allow_ec_snoop(true).unwrap();
                            // allow the EC to power me down
                            env.llio.allow_power_off(true).unwrap();
                            // now send the power off command
                            env.com.power_off_soc().unwrap();

                            log::info!("CMD: reboot in 3 seconds!");
                            // pause execution, nothing after this should be reachable
                            env.ticktimer.sleep_ms(3000).unwrap();
                            log::info!("CMD: if you can read this, reboot failed!");
                        }
                        write!(ret, "Cold boot request denied").unwrap();
                    }
                }
                "kill" => {
                    write!(ret, "Killing this device in 3 seconds.\nGoodbye cruel world!").unwrap();
                    xous::create_thread_1(kill_thread, 0).unwrap();
                }
                "killbounce" => {
                    if ((env.llio.adc_vbus().unwrap() as f64) * 0.005033) > 1.5 {
                        // if power is plugged in, deny powerdown request
                        write!(ret, "Unplug charging cable and try again.").unwrap();
                    } else {
                        write!(ret, "Killing this device in 3 seconds, then bouncing into ship mode\n").unwrap();
                        xous::create_thread_1(kill_thread, 1).unwrap();
                    }
                }
                "wfioff" => {
                    env.llio.wfi_override(true).unwrap();
                    write!(ret, "Overriding WFI signal, forcing always ON").unwrap();
                }
                "wfion" => {
                    env.llio.wfi_override(false).unwrap();
                    write!(ret, "Allowing WFI auto-control by kernel").unwrap();
                }
                "debugwfi" => {
                    env.llio.gpio_data_direction(0x3).unwrap(); // set bits 0 and 1 to output
                    env.llio.gpio_debug_powerdown(true).unwrap();
                    env.llio.gpio_debug_wakeup(true).unwrap();
                    write!(ret, "Connecting CRG powerdown to GPIO0, wakeup interrupt to GPIO1").unwrap();
                }
                _ =>  write!(ret, "{}", helpstring).unwrap(),
            }
        } else {
            write!(ret, "{}", helpstring).unwrap();
        }
        Ok(Some(ret))
    }
}
