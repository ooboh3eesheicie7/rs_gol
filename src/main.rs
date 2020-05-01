extern crate rand;
extern crate ctrlc;

// use rand::prelude::SliceRandom;
use rand::distributions::{Distribution, Uniform};
use std::io::{stdout, Write};
use crossterm::{
    ExecutableCommand,
    QueueableCommand,
    terminal,
    cursor, style::{Print, SetForegroundColor, SetBackgroundColor, ResetColor, Color}, Result
};
use std::{thread, time};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;


fn init_state(size: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let uni = Uniform::from(0..2);

    let mut vec: Vec<u8> = Vec::with_capacity(size);
    for _ in 0..vec.capacity() {
        vec.push(uni.sample(&mut rng));
    };
    return vec;
}

fn coords1d_to_coords2d(index: usize, size_y: usize) -> [usize; 2] {
    let quotient = index / size_y;
    let reminder = index - (quotient * size_y);
    return [quotient, reminder];
}

fn coords2d_to_coords1d(indexes: [usize; 2], size_y: usize) -> usize {
    return indexes[0] * size_y + indexes[1];
}

fn neighbours(index: usize, size_x: usize, size_y: usize) -> Vec<usize> {
    let indexes = coords1d_to_coords2d(index, size_y);
    let index_i = indexes[0] as i16;
    let index_j = indexes[1] as i16;
    let mut n: Vec<usize> = Vec::new();
    let delta: [i16; 3] = [-1, 0, 1];
    for di in delta.iter() {
        for dj in delta.iter() {
            if ((index_i == 0) & (*di == -1)) | ((index_i == size_x as i16 - 1) & (*di == 1))  {
                continue;
            }
            if ((index_j == 0) & (*dj == -1)) | ((index_j == size_y as i16 - 1) & (*dj == 1))  {
                continue;
            }
            if (*di == 0) & (*dj == 0) {
                continue;
            }
            n.push(coords2d_to_coords1d([(index_i + *di) as usize, (index_j + *dj) as usize], size_y));
        }
    }
    return n;
}

fn survive(state: &Vec<u8>, index: usize, size_x: usize, size_y: usize) -> bool {
    // let n = vec![
    //     (index) as i32 - 1,
    //     (index) as i32 + 1,
    //     (index) as i32 - (size_x) as i32,
    //     (index) as i32 - (size_x) as i32 - 1,
    //     (index) as i32 - (size_x) as i32 + 1,
    //     (index) as i32 + (size_x) as i32,
    //     (index) as i32 + (size_x) as i32 - 1,
    //     (index) as i32 + (size_x) as i32 + 1
    // ];
    let n = neighbours(index, size_x, size_y);
    let mut alive: u8 = 0;
    for x in n.iter() {
        if state[*x] == 1 {
            alive = alive + 1;
        }
    }

    // rules
    // Any live cell with two or three live neighbors survives.
    // Any dead cell with three live neighbors becomes a live cell.
    if (state[index] == 1) & (alive >= 2) & (alive <= 3) {
        return true;
    } else if (state[index] == 0) & (alive == 3) {
        return true;
    } else {
        return false;
    }
}

fn update(state: &Vec<u8>, size_x: usize, size_y: usize) -> Vec<u8> {
    let mut vec = vec![0; state.len()];
    for i in 0..vec.len() {
        if survive(state, i, size_x, size_y) {
            vec[i] = 1;
        }
    }
    return vec;
}

fn print_pix_term(state: &Vec<u8>, size_x: usize, size_y: usize, first: bool, refresh: bool) {
    let mut stdout = stdout();

    if first & refresh {
        // refresh term output like top/htop commands
        stdout.execute(terminal::Clear(terminal::ClearType::All));
    }

    if !refresh {
        stdout.queue(Print("+".repeat(size_y).to_string()));
        stdout.queue(Print("\n".to_string()));
    }

    let size = size_x * size_y;
    let quotient = size_x / 2;
    let reminder = size_x - (quotient * 2);
    for i in 0..(quotient + reminder) {
        for j in 0..size_y {
            if refresh {
                stdout.queue(cursor::MoveTo(j as u16, i as u16));
            }
            let index_top = i * 2 * size_y + j;
            let index_bot = (i * 2 + 1) * size_y + j;
            if (index_top < size) {
                if (index_bot < size) {
                    if (state[index_top] == 1) & (state[index_bot] == 1) {
                        stdout.queue(SetBackgroundColor(Color::Black));
                        stdout.queue(Print(" ".to_string()));
                    } else if state[index_top] == 1 {
                        stdout.queue(SetForegroundColor(Color::Black));
                        stdout.queue(Print("▀".to_string()));
                    } else if state[index_bot] == 1 {
                        stdout.queue(SetForegroundColor(Color::Black));
                        stdout.queue(Print("▄".to_string()));
                    } else {
                        stdout.queue(Print(" ".to_string()));
                    }
                } else {
                    if state[index_top] == 1 {
                        stdout.queue(SetForegroundColor(Color::Black));
                        stdout.queue(Print("▀".to_string()));
                    } else {
                        stdout.queue(Print(" ".to_string()));
                    }
                }
            } else {
                stdout.queue(Print(" ".to_string()));
            }

            stdout.queue(ResetColor);
        }
        if !refresh{
            stdout.queue(Print("\n".to_string()));
        }
    }
    if refresh {
        stdout.queue(cursor::MoveTo(0, (quotient + reminder) as u16));
    } else {
        stdout.queue(Print("+".repeat(size_y).to_string()));
        stdout.queue(Print("\n".to_string()));
    }
    stdout.flush();
}

fn print_numbers(state: &Vec<u8>, size_x: usize, size_y: usize, first: bool, refresh: bool) {
    let mut stdout = stdout();

    if first & refresh {
        // refresh term output like top/htop commands
        stdout.execute(terminal::Clear(terminal::ClearType::All));
    }
    if !refresh {
        stdout.queue(Print("+".repeat(size_y).to_string()));
        stdout.queue(Print("\n".to_string()));
    }

    let size = size_x * size_y;
    for i in 0..size_x {
        for j in 0..size_y {
            if refresh {
                stdout.queue(cursor::MoveTo(j as u16, i as u16));
            }

            // print!("{:?} ", state[coords2d_to_coords1d([i, j], size_y)]);
            stdout.queue(Print(state[coords2d_to_coords1d([i, j], size_y)].to_string()));
        }
        // println!("");
        stdout.queue(Print("\n".to_string()));
    }
    // println!("\n\n");
    if refresh {
        stdout.queue(cursor::MoveTo(0, size_x as u16));
    } else {
        stdout.queue(Print("+".repeat(size_y).to_string()));
        stdout.queue(Print("\n".to_string()));
    }
    stdout.flush();
}

fn main() -> Result<()> {
    // ctrl-C handler (stop the while loop)
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    let size_x: usize = 11;
    let size_y: usize = 20;
    let size = size_x * size_y;

    let mut state = init_state(size);
    // println!("{:?}", state);

    // special characters to mimic pixels
    // ▀
    // ▄

    let mut first: bool = true;
    let refresh: bool = true;
    let ascii: bool = true;

    while running.load(Ordering::SeqCst) {

        if ascii {
            print_pix_term(&state, size_x, size_y, first, refresh);
        } else {
            print_numbers(&state, size_x, size_y, first, refresh);
        }

        state = update(&state, size_x, size_y);

        thread::sleep(time::Duration::from_millis(500));
        if first {
            first = false;
        }
    }

    Ok(())
}
