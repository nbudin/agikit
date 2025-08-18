pub mod ibm_pcjr;

pub trait SoundNote {
    fn silent(start_time: u32, duration: u32) -> Self;
    fn duration(&self) -> u32;
    fn start_time(&self) -> u32;
    fn is_silent(&self) -> bool;
}
