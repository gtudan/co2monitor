mod zytemp;

fn main() {
    let mut device = zytemp::initialize();
    loop {
        let reading = zytemp::read_data(&mut device);
        println!("{:?}", reading);
    }
}
