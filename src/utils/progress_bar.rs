// progress_bar.rs

pub const PROGRESS_BAR_WIDTH: usize = 20;

pub fn inner_progress_bar(progress: f32, width: usize) -> String {
    let blocks = [' ', '▏', '▎', '▍', '▌', '▋', '▊', '▉', '█'];
    
    let clamped_progress = progress.clamp(0.0, 1.0);
    let full_blocks = (clamped_progress * width as f32).floor() as usize;

    let remainder_index = ((clamped_progress * width as f32 - full_blocks as f32) * (blocks.len() - 1) as f32).floor() as usize;

    let mut bar = String::new();
    bar.push_str(&"█".repeat(full_blocks));
    if full_blocks < width {
        bar.push(blocks[remainder_index]);
        bar.push_str(&" ".repeat(width - full_blocks - 1));
    }

    bar
}
