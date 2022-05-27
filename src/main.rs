pub mod composer;

use composer::*;

const VOL_MULTIPLIER: f32 = 0.5;

fn main() {

	let triangle_notes: Vec<Note> = vec![
		Note {
			pitch: 261.63,
			duration: 0.5,
		},
		Note {
			pitch: 293.66,
			duration: 0.5,
		},
		Note {
			pitch: 329.63,
			duration: 0.5,
		},
		Note {
			pitch: 349.23,
			duration: 0.5,
		},
		Note {
			pitch: 329.63,
			duration: 0.5,
		},
		Note {
			pitch: 293.66,
			duration: 0.5,
		},
	];

	let sine_notes: Vec<Note> = vec![
		Note {
			pitch: 261.63,
			duration: 1.5,
		},
		Note {
			pitch: 293.66,
			duration: 1.5,
		},	
	];

	let prototracks: Vec<ProtoTrack> = vec![
		ProtoTrack {
			instrument: Instruments::Triangle,
			notes: triangle_notes,
			tempo: 100,
		},
		ProtoTrack {
			instrument: Instruments::Sine,
			notes: sine_notes,
			tempo: 100,
		},
	];

	composer::play_song(prototracks);
}