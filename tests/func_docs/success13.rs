/// Some docs.
///
/// * `item`: some docs.
#[func]
pub fn notify(item: Vec<usize>) {
    println!("Breaking news! {}", item.summarize());
}
