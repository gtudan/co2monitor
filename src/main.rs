mod dostmann;

fn main() {
    let device = dostmann::initialize();
    dostmann::read_data(device);
}
