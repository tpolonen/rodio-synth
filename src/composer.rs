use core::time::Duration;
use rodio::{OutputStream, source::Source, Sink};

const VOL_MULTIPLIER: f32 = 0.5;
const SAMPLE_RATE: u32 = 44100;

struct Note {
	pitch: f32,
	duration: f32,
}

struct Track {
	instrument: &str,
	oscillator: WavetableOscillator,
	sink: Sink,
	volume: f32,
	notes: Vec<Note>,
	duration: f32,
}

#[derive(Clone)]
struct WavetableOscillator {
	sample_rate: u32,
	wave_table: Vec<f32>,
	index: f32,
	index_increment: f32,
}

// follows the oscillator code directly copied from a tutorial
// the basic idea is that each oscillator is an infinite source of a wave function:
// we fill each wavetable at the start of the main function and keep reusing it
// each time we create a new note
impl WavetableOscillator {

	fn new(sample_rate: u32, wave_table: Vec<f32>) -> WavetableOscillator {
		return WavetableOscillator {
			sample_rate,
			wave_table,
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

fn play_song(tracks: Vec<Track>) -> Result<char, &str> {

	//initialize wave tables

	let wave_table_size = 128;
	let mut sine_table: Vec<f32> = Vec::with_capacity(wave_table_size);
	let mut saw_table: Vec<f32> = Vec::with_capacity(wave_table_size);
	let mut square_table: Vec<f32> = Vec::with_capacity(wave_table_size);
	let mut triangle_table: Vec<f32> = Vec::with_capacity(wave_table_size);

	//fill each wave table
	for n in 0..wave_table_size {
		sine_table.push((2.0 * std::f32::consts::PI * n as f32 / wave_table_size as f32).sin());
	}

	for n in 0..wave_table_size {
		saw_table.push( -1.0 + (2.0 / wave_table_size as f32) * n as f32 );
	}

	for n in 0..wave_table_size {
		square_table.push(if (2.0 * std::f32::consts::PI * n as f32 / wave_table_size as f32).sin() >= 0.0 { 1.0 } else { -1.0});
	}

	for n in 0..wave_table_size {
		triangle_table.push( { if n < wave_table_size / 2 { 
			-1.0 + (2.0 / wave_table_size as f32) * n as f32 * 2.0 }
		else { 3.0 - (2.0 / wave_table_size as f32) * n as f32 * 2.0 }
		} );
	}

	//create infinite source from each wavetable
	let sine_oscillator = WavetableOscillator::new(SAMPLE_RATE, sine_table);
	let saw_oscillator = WavetableOscillator::new(SAMPLE_RATE, saw_table);
	let square_oscillator = WavetableOscillator::new(SAMPLE_RATE, square_table);
	let triangle_oscillator = WavetableOscillator::new(SAMPLE_RATE, triangle_table);

	//create output stream
	let (_stream, stream_handle) = OutputStream::try_default().unwrap();

	//track processing works like this:
	//0. clone appropriate oscillator for the track to use
	//1. create sink for the track -> sinks are automatically connected 
	//	to output stream
	//2. add each note to sink's buffer
	for track in tracks.iter_mut() {
		track.duration = 0.0;
		track.oscillator = {
			match track.instrument {
				"sine" => sine_oscillator.clone(),
				"saw" => saw_oscillator.clone(),
				"square" => square_oscillator.clone(),
				"triangle" => triangle_oscillator.clone(),
			}
		};
		track.sink = Sink::try_new(&stream_handle).unwrap();
		track.sink.pause();
		for note in track.notes.iter() {
			track.duration += notes.duration;
			track.oscillator.set_frequency(note.pitch);
			track.sink.append(track.oscillator.clone().take_duration(std::time::Duration::from_secs_f32(note.duration)));
		}
	}
	let ref mut longest_track : Track = None;
	let mut longest_duration : f32 = 0.0;
	for track in tracks.iter_mut() {
		track.sink.play();
		if track.duration > longest_duration {
			longest_track = &track;
			longest_duration = track.duration;
		}
	}

	longest_track.sink.sleep_until_end();

	Ok('👍')
}