extern crate rand;
extern crate crossbeam_channel;

use std::sync::{Arc, Mutex};
use rand::{Rng, thread_rng};
use std::{thread, time};
use std::thread::JoinHandle;

use crossbeam_channel::unbounded;

const NUM_GUESTS: i32 = 8;

fn main() {
    // We assign a thread to be our counter.
    // If the counter encounters a cupcake it leaves it
    // Otherwise the counter adds 1 to its count and requests another cupcake
    // Everyone else eats only the first cupcake they find
    println!("Problem 1:");
    let announcement = Arc::new(Mutex::new(false));

    let current_guest = Arc::new(Mutex::new(call_next_guest()));
    let cupcake = Arc::new(Mutex::new(1));

    // Create NUM_GUESTS guests labelled 1-NUM_GUESTS with an assigned counter thread
    let mut jobs: Vec<JoinHandle<()>> = Vec::new();

    // Clone a reference to each variable for the counter thread
    let current_guest_clone = Arc::clone(&current_guest);
    let cupcake_clone = Arc::clone(&cupcake);
    let announcement_clone = Arc::clone(&announcement);

    // Counter thread
    jobs.push(thread::spawn(move || {
        let mut counter = 0;
        loop {
            // Loop until an announcement has been made
            let mut announcement_lock = announcement_clone.lock().unwrap();
            if !*announcement_lock {
                let mut guest = current_guest_clone.lock().unwrap();
                // If it's the counter thread's turn in the labyrinth
                if *guest == 1 {
                    let mut cupcake_mut = cupcake_clone.lock().unwrap();
                    println!("Counter's turn.");
                    // If the cupcake isn't there count or make an announcement if possible
                    if *cupcake_mut == 0 {
                        counter += 1;
                        *cupcake_mut = 1;
                        println!("Counted guest!");
                        if counter == (NUM_GUESTS - 1) {
                            *announcement_lock = true;
                            println!("Announcement: all guests have had an opportunity at a cupcake");
                            break;
                        }
                    }
                    *guest = call_next_guest();
                }
            }
        }
    }));

    // Everyone else
    for i in 2..(NUM_GUESTS + 1) {
        // Clone references
        let current_guest_clone = Arc::clone(&current_guest);
        let cupcake_clone = Arc::clone(&cupcake);
        let announcement_clone = Arc::clone(&announcement);

        jobs.push(thread::spawn(move || {
            // Keep track of guest number and whether they ate a cupcake yet or not
            let guest_number = i.clone();
            let mut ate_cupcake = false;
            loop {
                // Loop until an announcement
                let announcement_lock = announcement_clone.lock().unwrap();
                if !*announcement_lock {
                    let mut guest = current_guest_clone.lock().unwrap();
                    let mut cupcake_mut = cupcake_clone.lock().unwrap();
                    // If it's our turn eat a cupcake only if we haven't already
                    if *guest == guest_number {
                        print!("Guest {}'s turn.", guest_number);
                        if *cupcake_mut == 1 && !ate_cupcake {
                            *cupcake_mut = 0;
                            ate_cupcake = true;
                            print!(" They ate the cupcake");
                        }
                        print!("\n");
                        *guest = call_next_guest();
                    }
                } else {
                    break;
                }
            }
        }));
    }

    // Wait for threads to finish
    for thread in jobs.into_iter(){
        thread.join().unwrap();
    }

    // Wait 5 seconds before running second problem
    thread::sleep(time::Duration::from_secs(5));
    println!("Problem 2:");

    // Problem 2: The Minotaur's Crystal Vase
    // Third strategy using a queue and message passing to let the next thread know.
    // This strategy's advantages are everyone is guaranteed to be able to go into the room
    // but it also requires everyone who wants to go in to wait in a queue instead of doing
    // something else

    // Keep track of if the room is occupied, who's in it, and a queue
    let room_occupied = Arc::new(Mutex::new(false));
    let current_guest = Arc::new(Mutex::new(0));
    let queue_init: Vec<i32> = (0..NUM_GUESTS).collect();
    let queue = Arc::new(Mutex::new(queue_init));

    // Keep track of threads and open up message channel to maintain queue and tell next in line it's their turn
    let mut jobs = Vec::new();
    let (tx, rx) = unbounded();

    for i in 0..NUM_GUESTS {
        // Clone a reference to variables for each thread
        let room_occupied_arc = Arc::clone(&room_occupied);
        let current_guest_arc = Arc::clone(&current_guest);
        let queue_arc = Arc::clone(&queue);
        let (s1, r1) = (tx.clone(), rx.clone());

        jobs.push(thread::spawn(move || {
            let queue_position = i.clone();
            loop {
                // If we receive a message
                if r1.recv().unwrap() {
                    let mut current_queue_head = current_guest_arc.lock().unwrap();
                    let mut queue_clone = queue_arc.lock().unwrap();

                    // Check if still in queue or not and skip if not
                    if (*queue_clone)[(*current_queue_head % NUM_GUESTS) as usize] == -1 {
                        *current_queue_head += 1;
                    }

                    // Check if this message is ours
                    if queue_position == (*current_queue_head  % NUM_GUESTS) {
                        // Occupy room for 1 second
                        println!("{} entered the room", queue_position);
                        let mut occupied = room_occupied_arc.lock().unwrap();
                        *occupied = true;
                        thread::sleep(time::Duration::from_secs(1));
                        *occupied = false;

                        // Tell next in queue that the room is open
                        *current_queue_head += 1;
                        s1.send(true).unwrap();

                        // 50% chance of leaving queue or reentering queue
                        let mut rng = thread_rng();
                        if rng.gen_bool(0.5){
                            println!("{} didn't reenter queue.", queue_position);
                            queue_clone[queue_position as usize] = -1;
                            break;
                        } else {
                            println!("{} reentered queue.", queue_position);
                        }
                    } else {
                        // This message wasn't for this thread, pass the message along
                        s1.send(true).unwrap();
                    }
                }
            }

        }))
    }
    // The room is open at the start
    tx.send(true).unwrap();

    // Wait until all threads are done
    for thread in jobs.into_iter() {
        thread.join().unwrap();
    }
}

fn call_next_guest() -> i32 {
    return thread_rng().gen_range(1..(NUM_GUESTS + 1));
}
