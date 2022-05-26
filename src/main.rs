use core::time::Duration;
use rodio::{OutputStream, source::Source, Sink};

#[derive(Clone)]
struct WavetableOscillator {
	sample_rate: u32,
	wave_table: Vec<f32>,
	index: f32,
	index_increment: f32,
}

impl WavetableOscillator {

	fn new(sample_rate: u32, wave_table: Vec<f32>) -> WavetableOscillator {
		return WavetableOscillator {
			sample_rate: sample_rate,
			wave_table: wave_table,
			index: 0.0,
			index_increment: 0.0,
		};
	}

	fn set_frequency(&mut self, frequency: f32) {
		self.index_increment = frequency * self.wave_table.len() as f32 
							   / self.sample_rate as f32;
	}

	fn get_sample(&mut self) -> f32 {
		let sample = self.lerp();
		self.index += self.index_increment;
		self.index %= self.wave_table.len() as f32;
		return sample;
	}

	fn lerp(&self) -> f32 {
		let truncated_index = self.index as usize;
		let next_index = (truncated_index + 1) % self.wave_table.len();
		
		let next_index_weight = self.index - truncated_index as f32;
		let truncated_index_weight = 1.0 - next_index_weight;

		return truncated_index_weight * self.wave_table[truncated_index] 
			   + next_index_weight * self.wave_table[next_index];
	}
} 

impl Source for WavetableOscillator {
	fn channels(&self) -> u16 {
		return 1;
	}

	fn sample_rate(&self) -> u32 {
		return self.sample_rate;
	}   

	fn current_frame_len(&self) -> Option<usize> {
		return None;
	}

	fn total_duration(&self) -> Option<Duration> {
		return None;
	}
}

impl Iterator for WavetableOscillator {
	type Item = f32;
	
	fn next(&mut self) -> Option<Self::Item> {
		return Some(self.get_sample());
	}
}

fn main() {
	let wave_table_size = 64;
	let mut sine_table: Vec<f32> = Vec::with_capacity(wave_table_size);
	let mut saw_table: Vec<f32> = Vec::with_capacity(wave_table_size);
	let mut square_table: Vec<f32> = Vec::with_capacity(wave_table_size);
	let mut triangle_table: Vec<f32> = Vec::with_capacity(wave_table_size);

	let saw_iterations = 100;
	let triangle_iterations = 100;

	for n in 1..wave_table_size {
		sine_table.push((2.0 * std::f32::consts::PI * n as f32 / wave_table_size as f32).sin());
	}

	for n in 0..wave_table_size {
		saw_table.push( {
			let mut sum = 0.0;
			for i in 1..saw_iterations {
				sum += (i as f32 * 2.0 * std::f32::consts::PI * n as f32 / wave_table_size as f32).sin() + 1.0 / (1.0 + i as f32);
				}
			sum
		} );
	}

	for n in 0..wave_table_size {
		square_table.push(if (2.0 * std::f32::consts::PI * n as f32 / wave_table_size as f32).sin() > 0.0 { 1.0 } else { -1.0});
	}

	for n in 0..wave_table_size {
		triangle_table.push( {
			let mut sum = 0.0;
			for i in (1..triangle_iterations).step_by(2) {
				sum += (if i == 3 { -1.0 } else { 1.0 }) * (i as f32 * 2.0 * std::f32::consts::PI * n as f32 / wave_table_size as f32).sin() + f32::powf(i as f32 + 2.0, -2.0)
				}
			sum
		} );
	}

	/* println!("sine");
	let mut sine_oscillator = WavetableOscillator::new(44100, sine_table);
	sine_oscillator.set_frequency(220.0);
	sine_oscillator.set_frequency(120.0);
	let sine_beat = sine_oscillator.clone().take_duration(std::time::Duration::from_secs(1));
	let (_stream, stream_handle) = OutputStream::try_default().unwrap();
	let sink = Sink::try_new(&stream_handle).unwrap();
	let mut saw_oscillator = WavetableOscillator::new(44100, saw_table);
	saw_oscillator.set_frequency(440.0);
	sink.append(saw_oscillator.take_duration(std::time::Duration::from_secs(1)));
	let new_beat = sine_beat.clone();
	sink.append(sine_oscillator.clone().take_duration(std::time::Duration::from_secs(1)));
	sink.append(sine_oscillator.clone().take_duration(std::time::Duration::from_secs(1)));
	sink.set_volume(0.5);
	sink.sleep_until_end(); */

	let (_stream, stream_handle) = OutputStream::try_default().unwrap();
	let sink = Sink::try_new(&stream_handle).unwrap();
	let sink2 = Sink::try_new(&stream_handle).unwrap();

	let mut sine_oscillator = WavetableOscillator::new(44100, sine_table);
	let mut triangle_oscillator = WavetableOscillator::new(44100, triangle_table);
	//C
	sine_oscillator.set_frequency(261.63);
	sink.append(sine_oscillator.clone().take_duration(std::time::Duration::from_secs_f32(0.5)));
	//D
	sine_oscillator.set_frequency(293.66);
	sink.append(sine_oscillator.clone().take_duration(std::time::Duration::from_secs_f32(0.5)));
	//E
	sine_oscillator.set_frequency(329.63);
	sink.append(sine_oscillator.clone().take_duration(std::time::Duration::from_secs_f32(0.5)));
	//F
	sine_oscillator.set_frequency(349.23);
	sink.append(sine_oscillator.clone().take_duration(std::time::Duration::from_secs_f32(0.5)));
	//E
	sine_oscillator.set_frequency(329.63);
	sink.append(sine_oscillator.clone().take_duration(std::time::Duration::from_secs_f32(0.5)));
	//D
	sine_oscillator.set_frequency(293.66);
	sink.append(sine_oscillator.clone().take_duration(std::time::Duration::from_secs_f32(0.5)));

	//C
	triangle_oscillator.set_frequency(261.63);
	sink2.append(triangle_oscillator.clone().take_duration(std::time::Duration::from_secs_f32(1.5)));
	//D
	triangle_oscillator.set_frequency(293.66);
	sink2.append(triangle_oscillator.clone().take_duration(std::time::Duration::from_secs_f32(1.5)));

	sink.set_volume(0.5);
	sink2.set_volume(0.5);
	sink.sleep_until_end();
	sink2.sleep_until_end();
	
//	std::thread::sleep(std::time::Duration::from_secs(2));
}