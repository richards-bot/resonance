/// A minor pentatonic scale frequencies across 3 octaves.
///
/// Notes: A, C, D, E, G — always harmonious regardless of which two collide.
pub const PENTATONIC_FREQS: &[f32] = &[
    // Octave 3
    220.00, // A3
    261.63, // C4
    293.66, // D4
    329.63, // E4
    392.00, // G4
    // Octave 4
    440.00, // A4
    523.25, // C5
    587.33, // D5
    659.25, // E5
    784.00, // G5
    // Octave 5
    880.00,  // A5
    1046.50, // C6
    1174.66, // D6
    1318.51, // E6
    1568.00, // G6
];
