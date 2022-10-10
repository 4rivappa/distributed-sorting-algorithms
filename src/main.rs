#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use std::time::Instant;
use rand::Rng;

use rand::thread_rng;
use rand::distributions::Uniform;

use std::collections::VecDeque;

struct Command {
    mode: String,
    from_id: i32,
    to_id: i32,
    val: i32
}


fn main(){
    let mut pe_count = 10;
    //let mut exit_flag = false;
    let mut exit_flag = Arc::new(Mutex::new(false));
    //let mut partial_order = ">=";
    let mut partial_order = Arc::new(Mutex::new(String::from("<=")));

    let commands_queue = Arc::new(Mutex::new(VecDeque::<Command>::new()));
    let cmd_queue_outer = Arc::clone(&commands_queue);
    let mut threads = vec![];

    for kkk in 0..pe_count{
        let commands_queue = Arc::clone(&commands_queue);
        let partial_order = Arc::clone(&partial_order);
        let exit_flag = Arc::clone(&exit_flag);

        let handle = thread::spawn(move || {
            // let commands_queue = Arc::clone(&commands_queue);
            // let partial_order = Arc::clone(&partial_order);
            // let exit_flag = Arc::clone(&exit_flag);

            let mut exit_thread: bool = false;
            let pe_id: i32 = kkk + 1;

            let mut rng = thread_rng();
            let mut value = rng.sample(Uniform::new(10i32, 15));
            // let value: i32 = rng.gen_range(1..10);
            println!("PE with id {} has started", pe_id);
            // handle exitflag loop
            
            while !exit_thread {
                let exit_flag = exit_flag.lock().unwrap();
                if *exit_flag == false {
                    drop(exit_flag);
                    let now = Instant::now();
                    //let five_seconds = Duration::from_secs(5);
                    //while now.elapsed().as_secs() <= five_seconds {
                    while now.elapsed().as_secs() <= 5 {
                        let mut cmd_queue = commands_queue.lock().unwrap();
                        if !cmd_queue.is_empty() {
                            let mut top_cmd_parent = cmd_queue.pop_front();
                            // let mut top_cmd = top_cmd_parent.take();
                            let mut top_cmd = top_cmd_parent.unwrap();
                            if top_cmd.mode == String::from("send") && top_cmd.from_id == pe_id {
                                cmd_queue.push_back(Command {
                                    mode: String::from("receive"),
                                    from_id: pe_id,
                                    to_id: top_cmd.to_id,
                                    val: value
                                });
                            } else if top_cmd.mode == String::from("receive") && top_cmd.to_id == pe_id {
                                let mut update_value : i32 = 0;
                                let partial_order = partial_order.lock().unwrap();
                                if *partial_order == String::from("<=") {
                                    if value < top_cmd.val {
                                        update_value = value;
                                        value = top_cmd.val;
                                    } else {
                                        update_value = top_cmd.val;
                                    }
                                } else if *partial_order == String::from(">=") {
                                    if value > top_cmd.val {
                                        update_value = value;
                                        value = top_cmd.val;
                                    } else {
                                        update_value = top_cmd.val;
                                    }
                                }
                                cmd_queue.push_back(Command {
                                    mode: String::from("update"),
                                    from_id: pe_id,
                                    to_id: top_cmd.from_id,
                                    val: update_value
                                });
                            } else if top_cmd.mode == String::from("update") && top_cmd.to_id == pe_id {
                                if value != top_cmd.val {
                                    value = top_cmd.val
                                }
                            } else if top_cmd.mode == String::from("print") && top_cmd.to_id == pe_id {
                                print!("{} ", value);
                            } else {
                                cmd_queue.push_front(top_cmd)
                            }
                            drop(cmd_queue);
                        } else {
                            drop(cmd_queue);
                        }
                    }
                } else {
                    drop(exit_flag);
                    exit_thread = true;
                }
            }


            println!("PE with id {} has stopped", pe_id);
        });
        threads.push(handle);
    }

    for i in 0..pe_count {
        let mod_val = (i+1)%2;
        let mut commands_queue_outer = cmd_queue_outer.lock().unwrap();
        for pe_id in 1..pe_count {
            if pe_id%2 == mod_val {
                commands_queue_outer.push_back(Command {
                    mode: String::from("send"),
                    from_id: pe_id,
                    to_id: pe_id+1,
                    val: 0
                });
            }
        }
        println!("Mod value with {} round has completed sending", mod_val);
        drop(commands_queue_outer);
        thread::sleep(Duration::from_millis(4000));
        let mut is_queue_empty = false;
        while is_queue_empty {
            let mut commands_queue_outer = cmd_queue_outer.lock().unwrap();
            if !commands_queue_outer.is_empty() {
                drop(commands_queue_outer);
                // time for threads to complete operations
                thread::sleep(Duration::from_millis(3000));
            } else {
                drop(commands_queue_outer);
                is_queue_empty = true;
            }
        }




        let mut commands_queue_outer = cmd_queue_outer.lock().unwrap();
        for pe_id in 1..pe_count {
            commands_queue_outer.push_back(Command {
                mode: String::from("print"),
                from_id: pe_id,
                to_id: pe_id,
                val: 0
            });
        }        
        drop(commands_queue_outer);
        thread::sleep(Duration::from_millis(4000));
        let mut is_queue_empty = false;
        while is_queue_empty {
            let mut commands_queue_outer = cmd_queue_outer.lock().unwrap();
            if !commands_queue_outer.is_empty() {
                drop(commands_queue_outer);
                // time for threads to complete operations
                thread::sleep(Duration::from_millis(3000));
            } else {
                drop(commands_queue_outer);
                is_queue_empty = true;
            }
        }
        println!("");
    }
    print!("{:?}", threads);
    let mut exit_flag_outer = exit_flag.lock().unwrap();
    *exit_flag_outer = true;

}