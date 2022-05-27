use core::time::Duration;
use rodio::{OutputStream, source::Source, Sink};
use rand::Rng;

const VOL_MULTIPLIER: f32 = 0.5;
const SAMPLE_RATE: u32 = 44100;

pub enum Instruments {
	Sine,
	Saw,
	Square,
	Triangle,
	Snare,
	Kick,
}

#[derive(Copy, Clone)]
pub struct Note {
	pub pitch: f32,
	pub duration: f32,
}

impl Note {
	fn new(pitch: f32, duration: f32) -> Note {
		return Note {
			pitch,
			duration,
		}
	}
}

pub struct ProtoTrack {
	pub instrument: Instruments,
	pub notes: Vec<Note>,
	pub tempo: u32,
}

impl ProtoTrack {
	fn new(instrument: Instruments) -> ProtoTrack {
		return ProtoTrack {
			instrument, 
			notes: Vec::new(),
			tempo: 0,
		}
	}
}

pub struct Track {
	pub oscillator: WavetableOscillator,
	pub sink: Sink,
	pub notes: Vec<Note>,
	pub volume: f32,
	pub duration: f32,
	pub tempo: u32,
}

impl Track {
	fn new(oscillator: WavetableOscillator, sink: Sink, notes: Vec<Note>, tempo: u32) -> Track {
		return Track {
			oscillator,
			sink,
			notes,
			volume: 1.0,
			duration: 0.0,
			tempo,
		}
	}
}

#[derive(Clone)]
pub struct WavetableOscillator {
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
	fn current_frame_len(&self) -> Option<usize> {
		return None;
	}

	fn channels(&self) -> u16 {
		return 1;
	}   

	fn sample_rate(&self) -> u32 {
		return self.sample_rate;
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

pub fn play_song(prototracks: Vec<ProtoTrack>) -> Result<char, ()> {

	let mut rng = rand::thread_rng();

	//initialize wave tables
	let wave_table_size = 128;
	let mut sine_table: Vec<f32> = Vec::with_capacity(wave_table_size);
	let mut saw_table: Vec<f32> = Vec::with_capacity(wave_table_size);
	let mut square_table: Vec<f32> = Vec::with_capacity(wave_table_size);
	let mut triangle_table: Vec<f32> = Vec::with_capacity(wave_table_size);
	let mut noise_table: Vec<f32> = Vec::with_capacity(wave_table_size);

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

	for _n in 0..wave_table_size {
		noise_table.push({
			(rng.gen::<f32>() * 2.00) - 1.00
		})
	}

	//create output stream
	let (_stream, stream_handle) = OutputStream::try_default().unwrap();

	//convert prototracks to tracks
	let mut tracks: Vec<Track> = Vec::new();

	for proto in prototracks.iter(){
		tracks.push(
			Track::new(WavetableOscillator::new(SAMPLE_RATE, match &proto.instrument {
				Instruments::Sine => sine_table.clone(),
				Instruments::Saw => saw_table.clone(),
				Instruments::Square => square_table.clone(),
				Instruments::Triangle => triangle_table.clone(),
				Instruments::Snare => noise_table.clone(),
				Instruments::Kick => noise_table.clone(),
			}), 
			Sink::try_new(&stream_handle).unwrap(), 
			proto.notes.clone(),
			proto.tempo)
		)
	}

	for track in tracks.iter_mut() {
		track.duration = 0.0;
		track.sink.pause();
		track.sink.set_volume(VOL_MULTIPLIER);
		for note in track.notes.iter() {
			track.duration += note.duration * (60.0 / track.tempo as f32);
			track.oscillator.set_frequency(note.pitch);
			track.sink.append(track.oscillator.clone().take_duration(std::time::Duration::from_secs_f32(note.duration * (60.0 / track.tempo as f32))));
		}
	}

	//we set each track to play at the same time; we also keep track on which track is the longest
	//so we don't stop executing program while it's still running.
	let mut longest_duration : f32 = 0.0;
	for track in tracks.iter_mut() {
		track.sink.play();
		if track.duration > longest_duration {longest_duration = track.duration}
	}

	std::thread::sleep(std::time::Duration::from_secs_f32(longest_duration));

	Ok('👍')
}