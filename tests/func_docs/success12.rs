/// Some docs.
///
/// # Arguments
/// * `item`: Some docs.
pub fn notify(item: &impl Summary) {
    println!("Breaking news! {}", item.summarize());
}
