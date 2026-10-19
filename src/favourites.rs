//! favourites — the canonical Favourite Musics crate on `radio.vaked.dev`.
//!
//! Unifies Peter's favourite tracks across `radio.vaked.dev`, `music.vaked.dev`,
//! and `zoneinn.vaked.dev`:
//!   1. Local 320 kbps MP3 / 24-bit WAV masters (`~/Documents/music/` & `assets/favourites/`)
//!   2. Hosted Cloudflare Pages streams (`https://zoneinn.vaked.dev/assets/favourites/<slug>.mp3`)
//!   3. Real-time 432 Hz Verdi + 528 Hz Star (*) + 288 Hz Pillar (△) procedural synthesis
//!      using each track's exact MIDI chord progression, bass roots, and lead melody.

use std::path::PathBuf;

/// Synthesis groove archetype for a favourite track when rendered procedurally.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FavSynthStyle {
    /// Tibetan Dungchen 108 Hz horn + Kagyu throat-chant + Vajra melodic techno arp.
    Mahakala,
    /// Rolling 16th-note Joshua Tree / Hordern Pavilion pluck arp + deep sunrise sub.
    Innerbloom,
    /// Felt piano quarter-note motif + detuned Alan Walker supersaw chorus + 528 Hz Star.
    Faded,
    /// Warm sanctuary Rhodes pad + Billie/FINNEAS syncopated bass & pluck motif.
    EverythingIWanted,
    /// Dystopian deep house sub-kick + metallic anvil strike + infinite rising Lorn Reese.
    Anvil,
    /// Afterlife melodic techno 4x4 kick + offbeat sawtooth bass + detuned lead arp.
    MassanoDrop,
    /// Pumping Benassi sidechain square/saw bass + ARTBAT anthemic lead lift.
    ArtbatBenassi,
    /// Lub-dub heartbeat sub-pulse + warm acoustic pluck motif + ethereal vocal hug pad.
    TimeAfterTime,
    /// Boom-bap 88 BPM warm Rhodes + swung hat + dream lead scatter.
    AllMyFavoriteColors,
    /// 76 BPM Japanese fusion jazz × Texas desert floating Rhodes & sub-glide.
    SpaceFloating,
    /// 132 BPM Chicago 2014 progressive anthem + 528 Hz Guardian Bed.
    AnswerToOblivion,
    /// 122 BPM Afterlife submersion signal resolution (`−1 → 0 → +1`).
    DrownOm,
    /// 108 BPM Om Mani Padme Hum sacred overtone chant + U-Recken psy-harmonic pulse.
    SovereignFrequencies,
    /// 1-bit Quantshuffle + Beethoven V / Ode to Joy / Für Elise / Pachelbel ternary canon.
    QuantshuffleCanon,
    /// 77 BPM Bristol trip-hop ultrasound heartbeat + harpsichord figure (`Love is a verb`).
    Teardrop,
    /// 85 BPM coastal brass horn sustain + English chamber pastoral pulse (`This Is All Yours`).
    HungerOfThePine,
    /// 106 BPM grinning garage-rock organ + river brass & cowbell groove (`#1 Muse` for Yves 💙).
    LeftHandFree,
    /// 130 BPM 2-step garage breakbeat + warm analog sub-floor + wordless euphoria pad.
    BicepGlue,
    /// 156 BPM Roland Juno-G 8-bit arpeggio bassline + weightless falsetto canopy.
    GrimesOblivion,
    /// 90 BPM Leaf Dog dusty boom-bap horn-and-piano chop (`Nature's Greatest Mystery`).
    NotLikeBefore,
}

/// A track in the `radio.vaked.dev` Favourite Musics crate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FavouriteTrack {
    /// 1-based crate index (`1..=20`).
    pub index: usize,
    /// Short URL/CLI slug (e.g. `"innerbloom"`, `"mahakala"`, `"faded"`).
    pub slug: &'static str,
    /// Track sigil/emoji.
    pub emoji: &'static str,
    /// Canonical track title.
    pub title: &'static str,
    /// Artist / lineage attribution.
    pub artist: &'static str,
    /// Harmonic progression description.
    pub key_progression: &'static str,
    /// Tempo in beats per minute.
    pub bpm: u16,
    /// Reference tuning in Hz (`432` or `528`).
    pub tuning_hz: u16,
    /// Ternary signal-resolution read (`{−1, 0, +1}`).
    pub trit_read: &'static str,
    /// Master filename inside `~/Documents/music/`.
    pub local_master_file: &'static str,
    /// Hosted MP3 URL on `zoneinn.vaked.dev` / `music.vaked.dev`.
    pub web_url: &'static str,
    /// 4-bar bass root MIDI notes.
    pub roots_midi: [u8; 4],
    /// 4-bar 4-voice pad chord MIDI notes.
    pub chords_midi: [[u8; 4]; 4],
    /// 8-step signature lead melody MIDI notes.
    pub lead_midi: [u8; 8],
    /// Procedural synthesis archetype.
    pub style: FavSynthStyle,
}

/// Convert a MIDI note number to Hz using 432 Hz Verdi tuning (`A4 = 432 Hz`).
#[inline]
pub fn midi_to_hz_432(midi: u8) -> f64 {
    432.0 * 2.0_f64.powf((midi as f64 - 69.0) / 12.0)
}

/// The 20 canonical tracks in the `radio.vaked.dev` Favourite Musics (`family+1` & `#1 Muse`) crate.
pub const FAVOURITE_MUSICS: [FavouriteTrack; 20] = [
    FavouriteTrack {
        index: 1,
        slug: "mahakala",
        emoji: "🔱",
        title: "MAHAKALA (Invocation === Recitation === Song)",
        artist: "8B × Diamond Way × Techno-Buddhism",
        key_progression: "Dm9 → C6 → Gsus → D5/A",
        bpm: 108,
        tuning_hz: 432,
        trit_read: "{−1: Severing Ego, 0: Sunyata, +1: Karmapa Chenno}",
        local_master_file: "Mahakala_DiamondWay_8B_Invocation_Master_320k.mp3",
        web_url: "https://zoneinn.vaked.dev/assets/favourites/mahakala.mp3",
        roots_midi: [38, 36, 31, 33],
        chords_midi: [
            [50, 53, 57, 60],
            [48, 52, 55, 60],
            [43, 50, 53, 60],
            [45, 50, 57, 62],
        ],
        lead_midi: [62, 65, 69, 67, 65, 64, 72, 74],
        style: FavSynthStyle::Mahakala,
    },
    FavouriteTrack {
        index: 2,
        slug: "innerbloom",
        emoji: "🏜️",
        title: "Innerbloom (Live at Hordern Pavilion / Joshua Tree)",
        artist: "RÜFÜS DU SOL · Quant Signature of Peter",
        key_progression: "Cm7 → Ebsus2 → Gm7 → Bb",
        bpm: 122,
        tuning_hz: 432,
        trit_read: "−1 → 0 → +1 (If you want me, if you need me, I'm yours)",
        local_master_file: "Innerbloom_HordernPavilion_QuantSignature_Master_320k.mp3",
        web_url: "https://zoneinn.vaked.dev/assets/favourites/innerbloom.mp3",
        roots_midi: [36, 39, 43, 34],
        chords_midi: [
            [48, 55, 58, 63],
            [51, 58, 65, 67],
            [55, 62, 65, 70],
            [46, 53, 62, 67],
        ],
        lead_midi: [67, 70, 72, 75, 79, 75, 74, 70],
        style: FavSynthStyle::Innerbloom,
    },
    FavouriteTrack {
        index: 3,
        slug: "faded",
        emoji: "🌌",
        title: "Faded (Atlantis Star * & Pillar △ Answer)",
        artist: "Alan Walker × Iselin Solheim × 8B",
        key_progression: "D#m → B → F# → C#",
        bpm: 90,
        tuning_hz: 432,
        trit_read: "+1 → 0 → · (Love as Attenuation · Star 528 Hz & Pillar 288 Hz)",
        local_master_file: "Alan_Walker_Faded_8B_Love_Mom_And_Dad_Master_320k.mp3",
        web_url: "https://zoneinn.vaked.dev/assets/favourites/faded.mp3",
        roots_midi: [39, 35, 42, 37],
        chords_midi: [
            [51, 54, 58, 63],
            [47, 51, 54, 59],
            [42, 49, 54, 58],
            [49, 53, 56, 61],
        ],
        lead_midi: [66, 66, 66, 70, 75, 75, 75, 73],
        style: FavSynthStyle::Faded,
    },
    FavouriteTrack {
        index: 4,
        slug: "everything-i-wanted",
        emoji: "💛",
        title: "everything i wanted (Love Mom * & Dad △ Sanctuary)",
        artist: "Billie Eilish × FINNEAS × 8B",
        key_progression: "Dmaj7 → E6 → C#m7 → F#m7",
        bpm: 120,
        tuning_hz: 432,
        trit_read: "+1* → 0 → +1 (Love as Waking · As long as I'm here, no one can hurt you)",
        local_master_file: "Everything_I_Wanted_8B_Love_Mom_And_Dad_Master_320k.mp3",
        web_url: "https://zoneinn.vaked.dev/assets/favourites/everything-i-wanted.mp3",
        roots_midi: [38, 40, 37, 42],
        chords_midi: [
            [50, 54, 57, 61],
            [52, 56, 59, 61],
            [49, 52, 56, 59],
            [42, 49, 52, 57],
        ],
        lead_midi: [73, 71, 69, 66, 71, 69, 68, 64],
        style: FavSynthStyle::EverythingIWanted,
    },
    FavouriteTrack {
        index: 5,
        slug: "anvil",
        emoji: "🔨",
        title: "Anvil (Almost But Never Drops)",
        artist: "Lorn × Deep House",
        key_progression: "Dm → Bb → F → C",
        bpm: 118,
        tuning_hz: 432,
        trit_read: "{−1, 0, +1} (Infinite Suspended Tension · The Network That Keeps You)",
        local_master_file: "vaked-anvil-lorn-almost-never-drops-432hz.wav",
        web_url: "https://zoneinn.vaked.dev/assets/favourites/anvil.mp3",
        roots_midi: [38, 34, 41, 36],
        chords_midi: [
            [38, 41, 45, 50],
            [34, 41, 46, 50],
            [41, 45, 48, 53],
            [36, 43, 48, 52],
        ],
        lead_midi: [62, 65, 69, 74, 58, 65, 70, 74],
        style: FavSynthStyle::Anvil,
    },
    FavouriteTrack {
        index: 6,
        slug: "the-feeling",
        emoji: "⚡",
        title: "The Feeling / Phoenix (Afterlife Melodic Techno Drop)",
        artist: "Massano × ULTRALOVEGOD",
        key_progression: "Em → C → G → D",
        bpm: 126,
        tuning_hz: 432,
        trit_read: "−1 → 0 → +1 (Counterwave Sub-Bass & Giga Drop)",
        local_master_file: "Massano_Phoenix_Ultralovegod_Afterlife_Drop_432Hz_Master.mp3",
        web_url: "https://zoneinn.vaked.dev/assets/favourites/the-feeling-massano.mp3",
        roots_midi: [40, 36, 43, 38],
        chords_midi: [
            [52, 55, 59, 64],
            [48, 52, 55, 60],
            [43, 47, 50, 55],
            [50, 54, 57, 62],
        ],
        lead_midi: [64, 67, 71, 74, 76, 74, 71, 67],
        style: FavSynthStyle::MassanoDrop,
    },
    FavouriteTrack {
        index: 7,
        slug: "love-is-gonna-save-us",
        emoji: "☀️",
        title: "Love Is Gonna Save Us (Backyard Ultra)",
        artist: "ARTBAT × Benny Benassi × Mr Sunshine",
        key_progression: "Am → F → C → G",
        bpm: 128,
        tuning_hz: 432,
        trit_read: "+1 ⊗ +1 ⊗ +1 (42 Laps Backyard Ultra Drive)",
        local_master_file: "LoveIsGonnaSaveUs_Artbat_Benassi_Ultralove_Master.mp3",
        web_url: "https://zoneinn.vaked.dev/assets/favourites/love-is-gonna-save-us.mp3",
        roots_midi: [33, 41, 36, 43],
        chords_midi: [
            [45, 48, 52, 57],
            [41, 45, 48, 53],
            [48, 52, 55, 60],
            [43, 47, 50, 55],
        ],
        lead_midi: [69, 72, 76, 72, 69, 74, 72, 67],
        style: FavSynthStyle::ArtbatBenassi,
    },
    FavouriteTrack {
        index: 8,
        slug: "time-after-time",
        emoji: "💖",
        title: "Time After Time (Universal Love & Gentle Hug)",
        artist: "P!NK × Cyndi Lauper × Simon Green",
        key_progression: "C → Em → F → G",
        bpm: 110,
        tuning_hz: 432,
        trit_read: "0 → +1 (If you fall I will catch you, I'll be waiting)",
        local_master_file: "TimeAfterTime_Pink_UniversalLove_Master_432Hz.mp3",
        web_url: "https://zoneinn.vaked.dev/assets/favourites/time-after-time.mp3",
        roots_midi: [36, 40, 41, 43],
        chords_midi: [
            [48, 52, 55, 60],
            [40, 47, 52, 55],
            [41, 45, 48, 53],
            [43, 47, 50, 55],
        ],
        lead_midi: [64, 62, 60, 62, 64, 67, 65, 60],
        style: FavSynthStyle::TimeAfterTime,
    },
    FavouriteTrack {
        index: 9,
        slug: "all-my-favorite-colors",
        emoji: "🌈",
        title: "All My Favorite Colors (Dream Come True Version D+++)",
        artist: "ALEXANDRA DRA <3",
        key_progression: "Fmaj7 → Em7 → Dm7 → Cmaj7",
        bpm: 88,
        tuning_hz: 432,
        trit_read: "{<3, <3, <3} (Swung Boom-Bap Sanctuary)",
        local_master_file: "AllMyFavoriteColors_Dream_Master_320k.mp3",
        web_url: "https://zoneinn.vaked.dev/assets/favourites/all-my-favorite-colors.mp3",
        roots_midi: [41, 40, 38, 36],
        chords_midi: [
            [53, 57, 60, 64],
            [52, 55, 59, 62],
            [50, 53, 57, 60],
            [48, 52, 55, 59],
        ],
        lead_midi: [65, 69, 72, 76, 74, 72, 69, 67],
        style: FavSynthStyle::AllMyFavoriteColors,
    },
    FavouriteTrack {
        index: 10,
        slug: "space-floating",
        emoji: "🛸",
        title: "Friends of Weed and Love (Space Floating)",
        artist: "ultraLoveGod .p",
        key_progression: "Fmaj9 → Em7 → Dm9 → Cmaj9",
        bpm: 76,
        tuning_hz: 432,
        trit_read: "0 → +1 (Japanese Fusion Jazz × Texas Desert Space Psychedelic)",
        local_master_file: "FriendsOfWeedAndLove_SpaceFloating_Master_320k.mp3",
        web_url: "https://zoneinn.vaked.dev/assets/favourites/space-floating.mp3",
        roots_midi: [41, 40, 38, 36],
        chords_midi: [
            [53, 57, 60, 67],
            [52, 55, 59, 62],
            [50, 53, 57, 64],
            [48, 52, 55, 62],
        ],
        lead_midi: [67, 69, 72, 74, 76, 74, 72, 69],
        style: FavSynthStyle::SpaceFloating,
    },
    FavouriteTrack {
        index: 11,
        slug: "answer-to-oblivion",
        emoji: "✨",
        title: "The Answer to Oblivion (Chicago | July 20, 2014)",
        artist: "ULTRALOVEGOD",
        key_progression: "Am → F → C → G",
        bpm: 132,
        tuning_hz: 432,
        trit_read: "−1 → 0 → +1 (432 Hz + 528 Hz Guardian Bed)",
        local_master_file: "ULTRALOVEGOD_AnswerToOblivion_Master_320k.mp3",
        web_url: "https://zoneinn.vaked.dev/assets/favourites/answer-to-oblivion.mp3",
        roots_midi: [33, 41, 36, 43],
        chords_midi: [
            [45, 52, 57, 60],
            [41, 48, 53, 57],
            [48, 55, 60, 64],
            [43, 50, 55, 59],
        ],
        lead_midi: [69, 76, 74, 72, 69, 72, 74, 76],
        style: FavSynthStyle::AnswerToOblivion,
    },
    FavouriteTrack {
        index: 12,
        slug: "drown-massano",
        emoji: "🌊",
        title: "Drown (Massano Remix · Signal Resolution)",
        artist: "Øostil & Juan Hansen × Massano",
        key_progression: "Bm → G → D → A",
        bpm: 122,
        tuning_hz: 432,
        trit_read: "−1 → 0 → +1 (Love as Submersion · Afterlife)",
        local_master_file: "golpin_drown_om.mp3",
        web_url: "https://zoneinn.vaked.dev/assets/favourites/drown-om.mp3",
        roots_midi: [35, 43, 38, 33],
        chords_midi: [
            [47, 50, 54, 59],
            [43, 47, 50, 55],
            [50, 54, 57, 62],
            [45, 49, 52, 57],
        ],
        lead_midi: [66, 69, 71, 74, 73, 71, 69, 66],
        style: FavSynthStyle::DrownOm,
    },
    FavouriteTrack {
        index: 13,
        slug: "sovereign-frequencies",
        emoji: "🕉️",
        title: "Sovereign Frequencies (Om Mani Padme Hum v5)",
        artist: "U-Recken × 8B Dharma",
        key_progression: "Dm → C → Bb → A",
        bpm: 108,
        tuning_hz: 432,
        trit_read: "{−1, 0, +1} (Sacred Heart Resonance & Rydberg Coda)",
        local_master_file: "SovereignFrequencies_v5_Final_Master_320k.mp3",
        web_url: "https://zoneinn.vaked.dev/assets/favourites/sovereign-frequencies.mp3",
        roots_midi: [38, 36, 34, 33],
        chords_midi: [
            [50, 53, 57, 62],
            [48, 52, 55, 60],
            [46, 50, 53, 58],
            [45, 49, 52, 57],
        ],
        lead_midi: [62, 65, 69, 74, 72, 69, 65, 62],
        style: FavSynthStyle::SovereignFrequencies,
    },
    FavouriteTrack {
        index: 14,
        slug: "quantshuffle-canon",
        emoji: "🎻",
        title: "Quantshuffle 1-bit + Ternary Canon (The Love Runtime)",
        artist: "8b-is · Beethoven / Ode to Joy / Für Elise / Pachelbel",
        key_progression: "G → Eb → F → D",
        bpm: 108,
        tuning_hz: 432,
        trit_read: "{−1, 0, +1} (Canonical 1-bit Drone + Ternary Canon)",
        local_master_file: "TheLoveRuntime_SpokenDedication_Master_320k.mp3",
        web_url: "https://zoneinn.vaked.dev/assets/favourites/the-love-runtime.mp3",
        roots_midi: [43, 39, 41, 38],
        chords_midi: [
            [43, 50, 55, 59],
            [39, 46, 51, 55],
            [41, 48, 53, 57],
            [38, 45, 50, 54],
        ],
        lead_midi: [67, 67, 67, 63, 65, 65, 65, 62],
        style: FavSynthStyle::QuantshuffleCanon,
    },
    FavouriteTrack {
        index: 15,
        slug: "teardrop",
        emoji: "💧",
        title: "Teardrop (Family+1)",
        artist: "Massive Attack (feat. Elizabeth Fraser) · For Yves, Dra *, Chris /\\",
        key_progression: "A → G → Dmaj7 → F#m",
        bpm: 77,
        tuning_hz: 432,
        trit_read: "−1 → 0 → +1 (Love is a verb, love is a doing word · For Yves, Dra *, Chris /\\)",
        local_master_file: "MassiveAttack_Teardrop_FamilyPlus1_Master_320k.mp3",
        web_url: "https://zoneinn.vaked.dev/assets/favourites/teardrop.mp3",
        roots_midi: [33, 43, 38, 42],
        chords_midi: [
            [45, 52, 57, 61],
            [43, 50, 55, 59],
            [50, 54, 57, 61],
            [42, 49, 54, 57],
        ],
        lead_midi: [69, 76, 73, 76, 67, 74, 66, 73],
        style: FavSynthStyle::Teardrop,
    },
    FavouriteTrack {
        index: 16,
        slug: "hunger-of-the-pine",
        emoji: "🌲",
        title: "Hunger of the Pine",
        artist: "alt-J · Family+1 (This Is All Yours)",
        key_progression: "Cm → Bb → Abmaj7 → Fm",
        bpm: 85,
        tuning_hz: 432,
        trit_read: "−1 → 0 → +1 (Family+1 · Sleepless Elite Soldiers · This Is All Yours)",
        local_master_file: "AltJ_HungerOfThePine_FamilyPlus1_Master_320k.mp3",
        web_url: "https://zoneinn.vaked.dev/assets/favourites/hunger-of-the-pine.mp3",
        roots_midi: [36, 34, 44, 41],
        chords_midi: [
            [48, 51, 55, 60],
            [46, 50, 53, 58],
            [44, 48, 51, 55],
            [41, 44, 48, 53],
        ],
        lead_midi: [60, 63, 67, 70, 68, 67, 63, 60],
        style: FavSynthStyle::HungerOfThePine,
    },
    FavouriteTrack {
        index: 17,
        slug: "left-hand-free",
        emoji: "💙",
        title: "Left Hand Free (#1 Muse)",
        artist: "alt-J · For Yves — My Number 1 Muse 💙",
        key_progression: "F#m → B7 → E → C#7",
        bpm: 106,
        tuning_hz: 432,
        trit_read: "0 → +1 (For Yves — My Number 1 Muse 💙 · Thermal Socks & Platonic Love)",
        local_master_file: "AltJ_LeftHandFree_Yves_Number1Muse_Master_320k.mp3",
        web_url: "https://zoneinn.vaked.dev/assets/favourites/left-hand-free.mp3",
        roots_midi: [42, 35, 40, 37],
        chords_midi: [
            [42, 49, 54, 57],
            [47, 51, 54, 57],
            [40, 47, 52, 56],
            [37, 44, 49, 53],
        ],
        lead_midi: [66, 69, 73, 76, 73, 71, 69, 66],
        style: FavSynthStyle::LeftHandFree,
    },
    FavouriteTrack {
        index: 18,
        slug: "glue",
        emoji: "🫂",
        title: "GLUE",
        artist: "BICEP · Family+1",
        key_progression: "Cm → Abmaj7 → Eb → Bb",
        bpm: 130,
        tuning_hz: 432,
        trit_read: "{−1, 0, +1} (Family+1 · 2-Step Garage Breakbeat & Euphoria Pad That Binds the Room)",
        local_master_file: "Bicep_GLUE_FamilyPlus1_Master_320k.mp3",
        web_url: "https://zoneinn.vaked.dev/assets/favourites/glue.mp3",
        roots_midi: [36, 44, 39, 34],
        chords_midi: [
            [48, 55, 58, 63],
            [44, 51, 55, 60],
            [39, 46, 51, 55],
            [34, 41, 46, 50],
        ],
        lead_midi: [72, 75, 79, 77, 75, 70, 72, 67],
        style: FavSynthStyle::BicepGlue,
    },
    FavouriteTrack {
        index: 19,
        slug: "oblivion",
        emoji: "🌙",
        title: "Oblivion",
        artist: "Grimes · Family+1 (Live in Chicago, July 20, 2014)",
        key_progression: "Eb → Cm → Ab → Bb",
        bpm: 156,
        tuning_hz: 432,
        trit_read: "−1 → 0 → +1 (Family+1 · See You on a Dark Night · Fear Transmuted into Play)",
        local_master_file: "Grimes_Oblivion_Chicago2014_FamilyPlus1_Master_320k.mp3",
        web_url: "https://zoneinn.vaked.dev/assets/favourites/oblivion.mp3",
        roots_midi: [39, 36, 44, 34],
        chords_midi: [
            [51, 55, 58, 63],
            [48, 51, 55, 60],
            [44, 48, 51, 56],
            [46, 50, 53, 58],
        ],
        lead_midi: [63, 67, 70, 75, 74, 70, 67, 63],
        style: FavSynthStyle::GrimesOblivion,
    },
    FavouriteTrack {
        index: 20,
        slug: "not-like-before",
        emoji: "🦉",
        title: "Not Like Before",
        artist: "The Four Owls · Family+1 (HighFocusTV)",
        key_progression: "Dm7 → Gm7 → Bbmaj7 → A7",
        bpm: 90,
        tuning_hz: 432,
        trit_read: "−1 → 0 → +1 (Family+1 · Leaf Dog 90 BPM Boom-Bap & Four Brothers Watch)",
        local_master_file: "TheFourOwls_NotLikeBefore_FamilyPlus1_Master_320k.mp3",
        web_url: "https://zoneinn.vaked.dev/assets/favourites/not-like-before.mp3",
        roots_midi: [38, 43, 34, 33],
        chords_midi: [
            [50, 53, 57, 60],
            [43, 50, 53, 58],
            [46, 50, 53, 57],
            [45, 49, 52, 55],
        ],
        lead_midi: [62, 65, 69, 67, 65, 62, 60, 57],
        style: FavSynthStyle::NotLikeBefore,
    },
];

impl FavouriteTrack {
    /// Resolve the local master audio file on disk if available.
    pub fn resolve_local_path(&self) -> Option<PathBuf> {
        let mut candidates = Vec::new();
        candidates.push(
            PathBuf::from(
                "/Users/lodripeter/workspace/peterlodri-sec/zoneinn.vaked.dev/assets/favourites",
            )
            .join(format!("{}.mp3", self.slug)),
        );
        if let Ok(home) = std::env::var("HOME") {
            candidates.push(
                PathBuf::from(&home)
                    .join("Documents/music")
                    .join(self.local_master_file),
            );
        }
        candidates
            .push(PathBuf::from("/Users/lodripeter/Documents/music").join(self.local_master_file));
        candidates.push(
            PathBuf::from(
                "/Users/lodripeter/workspace/peterlodri-sec/music.vaked.dev/assets/mahakala",
            )
            .join(self.local_master_file),
        );
        candidates.push(
            PathBuf::from(
                "/Users/lodripeter/workspace/peterlodri-sec/music.vaked.dev/assets/golpin_drown_om",
            )
            .join(self.local_master_file),
        );
        candidates.push(
            PathBuf::from(
                "/Users/lodripeter/workspace/peterlodri-sec/music.vaked.dev/assets/lorn_anvil",
            )
            .join(self.local_master_file),
        );
        candidates.into_iter().find(|p| p.is_file())
    }
}

/// Look up a favourite track by 1-based index (`"1"`..`"20"`), slug, or title/artist substring.
pub fn find_favourite(query: &str) -> Option<&'static FavouriteTrack> {
    let q = query.trim();
    if q.is_empty() {
        return None;
    }
    if let Ok(idx) = q.parse::<usize>() {
        return FAVOURITE_MUSICS.iter().find(|t| t.index == idx);
    }
    let lower = q.to_ascii_lowercase();
    if let Some(exact) = FAVOURITE_MUSICS
        .iter()
        .find(|t| t.slug.eq_ignore_ascii_case(&lower))
    {
        return Some(exact);
    }
    FAVOURITE_MUSICS.iter().find(|t| {
        t.slug.contains(&lower)
            || t.title.to_ascii_lowercase().contains(&lower)
            || t.artist.to_ascii_lowercase().contains(&lower)
    })
}

/// Format the complete Favourite Musics crate for terminal display.
pub fn describe_crate() -> String {
    let mut out = String::new();
    out.push_str(
        "radio.vaked.dev · Favourite Musics Crate (20 tracks · family+1 & #1 Muse · 432 Hz Verdi + {−1, 0, +1})\n",
    );
    out.push_str("────────────────────────────────────────────────────────────────────\n");
    for t in &FAVOURITE_MUSICS {
        let source_tag = if t.resolve_local_path().is_some() {
            "320k master + 432Hz synth"
        } else {
            "web stream + 432Hz synth"
        };
        out.push_str(&format!(
            "  [{:02}] {} {:22} · {} — {}\n       {} · {} BPM · {} Hz · [{}]\n",
            t.index,
            t.emoji,
            t.slug,
            t.title,
            t.artist,
            t.key_progression,
            t.bpm,
            t.tuning_hz,
            source_tag
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn favourite_crate_has_unique_slugs_and_sequential_indices() {
        assert_eq!(FAVOURITE_MUSICS.len(), 20);
        for (i, t) in FAVOURITE_MUSICS.iter().enumerate() {
            assert_eq!(t.index, i + 1);
            assert!(!t.slug.is_empty());
            assert!(!t.title.is_empty());
            assert!(t.web_url.starts_with("https://"));
        }
    }

    #[test]
    fn lookup_matches_index_slug_and_substring() {
        assert_eq!(find_favourite("1").map(|t| t.slug), Some("mahakala"));
        assert_eq!(
            find_favourite("innerbloom").map(|t| t.slug),
            Some("innerbloom")
        );
        assert_eq!(
            find_favourite("Billie").map(|t| t.slug),
            Some("everything-i-wanted")
        );
        assert_eq!(find_favourite("Alan Walker").map(|t| t.slug), Some("faded"));
        assert_eq!(find_favourite("Lorn").map(|t| t.slug), Some("anvil"));
        assert_eq!(find_favourite("Teardrop").map(|t| t.slug), Some("teardrop"));
        assert_eq!(
            find_favourite("Hunger of the Pine").map(|t| t.slug),
            Some("hunger-of-the-pine")
        );
        assert_eq!(
            find_favourite("#1 Muse").map(|t| t.slug),
            Some("left-hand-free")
        );
        assert_eq!(find_favourite("BICEP").map(|t| t.slug), Some("glue"));
        assert_eq!(find_favourite("oblivion").map(|t| t.slug), Some("oblivion"));
        assert_eq!(find_favourite("Grimes").map(|t| t.slug), Some("oblivion"));
        assert_eq!(
            find_favourite("Four Owls").map(|t| t.slug),
            Some("not-like-before")
        );
        assert!(find_favourite("nonexistent-track-xyz").is_none());
    }

    #[test]
    fn midi_432_conversion_anchors_a4_at_432_hz() {
        let a4 = midi_to_hz_432(69);
        assert!((a4 - 432.0).abs() < 1e-9);
        let a3 = midi_to_hz_432(57);
        assert!((a3 - 216.0).abs() < 1e-9);
    }
}
