pub fn clamp_u32(value: u32, min: u32, max: u32) -> u32 {
    if value < min {
        return min;
    }
    
    if value > max {
        return max;
    }

    return value;
}