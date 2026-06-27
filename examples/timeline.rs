use nelo::timeline::Timeline;

fn main() {
    let t1 = Timeline::constant(5);
    let t2 = Timeline::dynamic(|x| x * x);

    for t in 0..5 {
        let v1 = t1.sample(t as f64);
        let v2 = t2.sample(t as f64);
        println!("t={t} => t1={v1} & t2={v2}");
    }
}
