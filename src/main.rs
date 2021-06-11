use std::time::Instant;
use std::fs::{File, remove_file, OpenOptions};
use rand::Rng;
use std::io::{Write, Seek, SeekFrom, Read};

fn measure_cpu(count: u64, _: ()) -> i64 {
    use sha256::digest;
    let mut result = 0;
    for index in 0 .. count {
        let input = "what should I do but tend upon the hours, and times of your desire";
        let val = digest(input);
        assert_eq!(val, "9b4d38fd42c985baec11564a84366de0cbd26d3425ec4ce1266e26b7b951ac08");
        result += val.as_bytes()[(index % 64) as usize] as i64;
    }
    result
}

fn create_file(size: u64) -> String {
    let file_name = format!("file_{}.dat", rand::thread_rng().gen::<u32>());
    let file = File::create(file_name.clone()).unwrap();
    file.set_len(size).unwrap();
    file_name
}

fn create_file_and_write(size: u64) -> String {
    let file_name = create_file(size);
    measure_io_write_seq(size, file_name.clone());
    file_name
}

fn measure_io_write_seq(count: u64, file_name: String) -> i64 {
    let mut file = OpenOptions::new().write(true).create(false).open(file_name).unwrap();
    for i in 0 .. count {
        let buf = [i as u8];
        file.write(&buf).unwrap();
    }
    0
}

fn measure_io_write_random(count: u64, file_name: String) -> i64 {
    let mut file = OpenOptions::new().write(true).create(false).open(file_name).unwrap();
    for i in 0 .. count {
        let buf = [i as u8];
        let position = (rand::thread_rng().gen::<u64>()) % count;
        file.seek(SeekFrom::Start(position)).unwrap();
        file.write(&buf).unwrap();
    }
    0
}

fn measure_io_read_seq(count: u64, file_name: String) -> i64 {
    let mut file = File::open(file_name).unwrap();
    for _i in 0 .. count {
        let mut buf = [0];
        file.read(&mut buf).unwrap();
    }
    0
}

fn measure_io_read_random(count: u64, file_name: String) -> i64 {
    let mut file = File::open(file_name).unwrap();
    for _i in 0 .. count {
        let mut buf = [0];
        let position = (rand::thread_rng().gen::<u64>()) % count;
        file.seek(SeekFrom::Start(position)).unwrap();
        file.read(&mut buf).unwrap();
    }
    0
}

fn cleanup_file(file_name: String) {
    remove_file(file_name).unwrap();
}

#[used]
static mut SINK: i64 = 0;

fn measure_operation<F1: FnOnce(u64) -> T, F2: FnOnce(u64, T) -> i64, F3: FnOnce(T) -> (), T: Clone>(
    count: u64,
    prepare: F1,
    op: F2,
    cleanup: F3
) -> u128 {
    let prepared = prepare(count);
    let start = Instant::now();
    let value = op(count, prepared.clone());
    let result = start.elapsed().as_nanos();
    unsafe {
        SINK = value;
    }
    cleanup(prepared);
    result
}


fn main() {
    #[cfg(debug_assertions)]
    println!("WARNING: calibrator must run in release mode to provide accurate results!");
    const CPU_REPEATS: u64 = 10000;
    let cpu = measure_operation(CPU_REPEATS, |_| (), measure_cpu, |()| ());
    println!("CPU executes SHA256 in {} ns", (cpu as u64) / CPU_REPEATS);
    const IO_SIZE: u64 = 1 * 1000 * 1000;
    let io_write_seq = measure_operation(IO_SIZE,
                                     create_file, measure_io_write_seq, cleanup_file);
    println!("IO sequential write {} ns per byte", (io_write_seq as u64) / IO_SIZE);
    let io_write_random = measure_operation(IO_SIZE,
                                         create_file, measure_io_write_random, cleanup_file);
    println!("IO random write {} ns per byte", (io_write_random as u64)/ IO_SIZE);
    let io_read_seq = measure_operation(IO_SIZE,
                                     create_file_and_write, measure_io_read_seq, cleanup_file);
    println!("IO sequential read {} ns per byte", (io_read_seq as u64) / IO_SIZE);
    let io_read_random = measure_operation(IO_SIZE,
                                         create_file_and_write, measure_io_read_random, cleanup_file);
    println!("IO random read {} ns per byte", (io_read_random as u64) / IO_SIZE);
}
