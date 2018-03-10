mod zytemp;

fn main() {
    let device = zytemp::initialize();
    zytemp::read_data(device);
}
