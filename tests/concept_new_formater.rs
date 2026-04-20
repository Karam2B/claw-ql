pub trait Format<F: Formater> {
    fn format(&self, formater: &mut F);
}

pub trait Formater {}

pub trait FormatingNumbers: Formater {
    fn format_i64(&mut self, value: i64);
    fn format_f64(&mut self, value: f64);
    fn format_u64(&mut self, value: u64);
    fn format_usize(&mut self, value: usize);
    fn format_isize(&mut self, value: isize);
    fn format_u8(&mut self, value: u8);
    fn format_u16(&mut self, value: u16);
    fn format_u32(&mut self, value: u32);
}

impl<F: FormatingNumbers> Format<F> for i64 {
    fn format(&self, formater: &mut F) {
        formater.format_i64(*self);
    }
}

pub struct DisplayFormater(String);

impl Formater for DisplayFormater {}

impl FormatingNumbers for DisplayFormater {
    fn format_i64(&mut self, value: i64) {
        self.0.push_str(&value.to_string());
    }
    fn format_f64(&mut self, value: f64) {
        self.0.push_str(&value.to_string());
    }
    fn format_u64(&mut self, value: u64) {
        self.0.push_str(&value.to_string());
    }
    fn format_usize(&mut self, value: usize) {
        self.0.push_str(&value.to_string());
    }
    fn format_isize(&mut self, value: isize) {
        self.0.push_str(&value.to_string());
    }
    fn format_u8(&mut self, value: u8) {
        self.0.push_str(&value.to_string());
    }
    fn format_u16(&mut self, value: u16) {
        self.0.push_str(&value.to_string());
    }
    fn format_u32(&mut self, value: u32) {
        self.0.push_str(&value.to_string());
    }
}
