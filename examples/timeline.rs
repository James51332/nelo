use nelo::timeline::Timeline;

fn main() {
    let t1 = Timeline::constant(5);
    let t2 = Timeline::dynamic(|x| x * x).with_length(2.0);

    println!(
        "Created two timelines t1 (len: {:?}) and t2 (len: {:?})",
        t1.length(),
        t2.length()
    );
    for t in 0..5 {
        let v1 = t1.sample(t as f32);
        let v2 = t2.sample(t as f32);
        println!("t={t} => t1={v1} & t2={v2}");
    }
}
