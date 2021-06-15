use itertools::Itertools;
use rand::Rng;
use std::collections::HashMap;
use std::fs::{remove_file, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::time::Instant;

fn measure_cpu(count: u64, _: ()) -> i64 {
    use sha256::digest;
    let mut result = 0;
    for index in 0..count {
        let input = "what should I do but tend upon the hours, and times of your desire";
        let val = digest(input);
        assert_eq!(
            val,
            "9b4d38fd42c985baec11564a84366de0cbd26d3425ec4ce1266e26b7b951ac08"
        );
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
    let mut file = OpenOptions::new()
        .write(true)
        .create(false)
        .open(file_name)
        .unwrap();
    for i in 0..count {
        let buf = [i as u8];
        file.write(&buf).unwrap();
    }
    0
}

fn measure_io_write_random(count: u64, file_name: String) -> i64 {
    let mut file = OpenOptions::new()
        .write(true)
        .create(false)
        .open(file_name)
        .unwrap();
    for i in 0..count {
        let buf = [i as u8];
        let position = (rand::thread_rng().gen::<u64>()) % count;
        file.seek(SeekFrom::Start(position)).unwrap();
        file.write(&buf).unwrap();
    }
    0
}

fn measure_io_read_seq(count: u64, file_name: String) -> i64 {
    let mut file = File::open(file_name).unwrap();
    for _i in 0..count {
        let mut buf = [0];
        file.read(&mut buf).unwrap();
    }
    0
}

fn measure_io_read_random(count: u64, file_name: String) -> i64 {
    let mut file = File::open(file_name).unwrap();
    for _i in 0..count {
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

fn measure_operation<
    F1: FnOnce(u64) -> T,
    F2: FnOnce(u64, T) -> i64,
    F3: FnOnce(T) -> (),
    T: Clone,
>(
    count: u64,
    prepare: F1,
    op: F2,
    cleanup: F3,
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

fn output_stdout(kind: String, output_data: &HashMap<(String, u64), u128>) {
    for (key, value) in output_data.iter().sorted() {
        println!(
            "{}: scale {}: {} ns per {}",
            kind,
            key.1,
            value / (key.1 as u128),
            key.0,
        )
    }
}

fn output_gnuplot(file: String, kind: String, output_data: &HashMap<(String, u64), u128>) {
    let mut file = File::create(format!("{}_{}", kind, file)).unwrap();
    let mut data = HashMap::<u64, Vec<(u128, String)>>::new();
    for (key, value) in output_data.iter().sorted() {
        let v = data.entry(key.1).or_insert_with(|| Vec::new());
        v.push((*value, key.clone().0));
    }
    let mut index = 0;
    for (key, value) in data {
        //println!("{} {:?}", key, value);
        if index == 0 {
            for (_, comment) in value.clone() {
                write!(file, "\"{}\" ", comment).unwrap();
            }
            write!(file, "\n").unwrap();
        }
        index += 1;
        write!(
            file,
            "{} {}\n",
            key,
            value
                .iter()
                .map(|(y, _comment)| y.to_string())
                .collect::<Vec<String>>()
                .join(" ")
        )
        .unwrap();
    }
}

fn finish_output(out: String, output_data: &HashMap<(String, u64), u128>, kind: String) {
    let dest = out.split(':').collect::<Vec<&str>>();
    match dest[0] {
        "gnuplot" => output_gnuplot(dest[1].to_string(), kind, output_data),
        "stdout" => output_stdout(kind, output_data),
        _ => panic!("Unknown output: {}", out),
    }
}

fn output(kind: String, count: u64, value: u128, output_data: &mut HashMap<(String, u64), u128>) {
    let key = (kind, count);
    output_data.insert(key, value);
}

fn parse_seq_or(arg: String, default: u64) -> Vec<u64> {
    if arg.is_empty() {
        vec![default]
    } else {
        arg.split(',').map(|x| x.parse::<u64>().unwrap()).collect()
    }
}

fn main() {
    use structopt::StructOpt;
    #[derive(StructOpt)]
    struct Cli {
        #[structopt(short = "c", long = "cpu-iterations", default_value = "10000")]
        num_cpu_iterations: u64,
        #[structopt(short = "i", long = "io-size", default_value = "100000")]
        io_size: u64,
        #[structopt(short = "o", long = "output", default_value = "stdout")]
        output: String,
        #[structopt(long = "cpu-range", default_value = "")]
        cpu_range: String,
        #[structopt(long = "io-range", default_value = "")]
        io_range: String,
    }
    let args = Cli::from_args();
    let mut output_data_io: HashMap<(String, u64), u128> = HashMap::new();
    let mut output_data_cpu: HashMap<(String, u64), u128> = HashMap::new();

    #[cfg(debug_assertions)]
    println!("WARNING: calibrator must run in release mode to provide accurate results!");
    let cpu_range = parse_seq_or(args.cpu_range, args.num_cpu_iterations);
    for count in cpu_range {
        let cpu = measure_operation(count, |_| (), measure_cpu, |()| ());
        output("SHA256".to_string(), count, cpu, &mut output_data_cpu);
    }
    let io_range = parse_seq_or(args.io_range, args.io_size);
    for count in io_range {
        let io_write_seq =
            measure_operation(count, create_file, measure_io_write_seq, cleanup_file);
        output(
            "IO sequential write".to_string(),
            count,
            io_write_seq,
            &mut output_data_io,
        );
        let io_write_random =
            measure_operation(count, create_file, measure_io_write_random, cleanup_file);
        output(
            "IO random write".to_string(),
            count,
            io_write_random,
            &mut output_data_io,
        );
        let io_read_seq = measure_operation(
            count,
            create_file_and_write,
            measure_io_read_seq,
            cleanup_file,
        );
        output(
            "IO sequential read".to_string(),
            count,
            io_read_seq,
            &mut output_data_io,
        );
        let io_read_random = measure_operation(
            count,
            create_file_and_write,
            measure_io_read_random,
            cleanup_file,
        );
        output(
            "IO random read".to_string(),
            count,
            io_read_random,
            &mut output_data_io,
        );
    }
    finish_output(args.output.clone(), &output_data_cpu, "cpu".to_string());
    finish_output(args.output, &output_data_io, "io".to_string());
}
