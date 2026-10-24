// ╔══════════════════════════════════════════════════════════════════════════╗
// ║  PROGRESSIVE_DEBATIC / types.rs                                          ║
// ║  GJATA LEGACY™ — AIGovern-Cyber Quantum                                         ║
// ║                                                                          ║
// ║  ROLI: Kontratat e tipeve për modulin PD (cognitive excavation layer).  ║
// ║                                                                          ║
// ║  ÇFARË ËSHTË PD:                                                         ║
// ║    — NUK është debater/argument engine.                                 ║
// ║    — Është shtresë gërmimi kognitiv: gjen "gjëgjëzën" e fshehur në       ║
// ║      mendimin e shfaqur, gërmon trace-in, dhe — kur lind një GeniusSignal║
// ║      — dorëzon material tek PIM.                                         ║
// ║    — Kundërshton VETËM kur mungon baza reale (pressure budget).          ║
// ║                                                                          ║
// ║  Përshtatur në arkitekturën sovrane: ZERO if/else (match/branchless),    ║
// ║  pa varësi të jashtme, id-të me FNV-1a (hash-i real i sistemit).         ║
// ╚══════════════════════════════════════════════════════════════════════════╝

/// id 12-shenjësh heksadecimal nga FNV-1a (deterministik, hash-i real i sistemit).
pub fn pd_id(parts: &[&[u8]]) -> String {
    // Përziej pjesët në mënyrë branchless (fold mbi FNV).
    let h = parts.iter().fold(0xcbf29ce484222325u64, |acc, p| {
        crate::fnv1a64(*p) ^ acc.wrapping_mul(1099511628211)
    });
    format!("{:012x}", h & 0xFFFF_FFFF_FFFF)
}

// ─────────────────────────────────────────────────────────────────────────────
// DebateMode — 4 mënyrat e angazhimit
// ─────────────────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebateMode {
    Intellect,  // nevoja praktike — shtyn lehtë
    Philosophy, // intuitë pa strukturë — PD ndërton scaffold
    Scientific, // koncepti lidhet me matje — kërkon kauzalitet/invariantë
    Novel,      // zona e frontierit — kundërshtim shumë i butë
}

impl DebateMode {
    pub fn label(self) -> &'static str {
        match self {
            DebateMode::Intellect => "INTELLECT",
            DebateMode::Philosophy => "PHILOSOPHY",
            DebateMode::Scientific => "SCIENTIFIC",
            DebateMode::Novel => "NOVEL",
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// CognitiveSignal — çdo mesazh i shfaqur, i procesuar gjatë ingest
// ─────────────────────────────────────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct CognitiveSignal {
    pub signal_id: String,
    pub turn_index: u32,
    pub raw_content: String,
    pub content_length: u32,
    pub idea_density: u32, // fixed mass [0..10_000]
    pub energy_level: u32, // fixed mass [0..10_000]
    pub concept_refs: Vec<String>,
    pub contradicts: Vec<String>,
    pub returns_to: Vec<String>,
    pub timestamp: u64,
}

// ─────────────────────────────────────────────────────────────────────────────
// EpistemicTrace — gjurma e akumuluar gjatë sesionit
// ─────────────────────────────────────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct EpistemicTrace {
    pub trace_id: String,
    pub session_id: String,
    pub signals: Vec<CognitiveSignal>,
    pub total_turns: u32,
    pub dominant_concept: Option<String>,
    pub concept_map: std::collections::HashMap<String, u32>,
    pub energy_trajectory: Vec<u32>,
    pub idea_evolution: Vec<u32>,
    pub accumulated_mass: u32,
    pub structural_coherence: u32,
    pub implicit_primitives: Vec<String>,
    pub contradiction_pairs: Vec<(String, String)>,
    pub territory: String,
    pub started_at: u64,
    pub last_updated_at: u64,
}

impl EpistemicTrace {
    pub fn new(session_id: &str, territory: &str, started_at: u64) -> Self {
        EpistemicTrace {
            trace_id: pd_id(&[session_id.as_bytes(), b"trace"]),
            session_id: session_id.to_string(),
            signals: Vec::new(),
            total_turns: 0,
            dominant_concept: None,
            concept_map: std::collections::HashMap::new(),
            energy_trajectory: Vec::new(),
            idea_evolution: Vec::new(),
            accumulated_mass: 0,
            structural_coherence: 0,
            implicit_primitives: Vec::new(),
            contradiction_pairs: Vec::new(),
            territory: territory.to_string(),
            started_at,
            last_updated_at: started_at,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// GeniusSignal — momenti kur sistemi "ndjen" diçka reale (3 sinjale)
// ─────────────────────────────────────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct GeniusSignal {
    pub signal_id: String,
    pub session_id: String,
    pub returns_to_idea: bool,
    pub energy_increasing: bool,
    pub concept_gaining_shape: bool,
    pub returns_score: u32,
    pub energy_score: u32,
    pub shape_score: u32,
    pub genius_score: u32,
    pub detected_concept: String,
    pub detected_at_turn: u32,
    pub trace_ref: String,
    pub timestamp: u64,
}

// ─────────────────────────────────────────────────────────────────────────────
// PressureBudget — sa mund të shtyjë PD para se të tërhiqet
// ─────────────────────────────────────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct PressureBudget {
    pub challenges_issued: u32,
    pub current_mode: DebateMode,
    pub budget_exhausted: bool,
    pub user_resistance_detected: bool,
}

impl PressureBudget {
    // Limitet absolute sipas mode-it (nga pseudo).
    pub const MAX_INTELLECT: u32 = 2;
    pub const MAX_PHILOSOPHY: u32 = 3;
    pub const MAX_SCIENTIFIC: u32 = 4;
    pub const MAX_NOVEL: u32 = 2;

    pub fn new(mode: DebateMode) -> Self {
        PressureBudget {
            challenges_issued: 0,
            current_mode: mode,
            budget_exhausted: false,
            user_resistance_detected: false,
        }
    }

    /// max_for — limiti i kundërshtimeve për mode-in aktual. Zero if — match.
    pub fn max_for(mode: DebateMode) -> u32 {
        match mode {
            DebateMode::Intellect => Self::MAX_INTELLECT,
            DebateMode::Philosophy => Self::MAX_PHILOSOPHY,
            DebateMode::Scientific => Self::MAX_SCIENTIFIC,
            DebateMode::Novel => Self::MAX_NOVEL,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// PdResponseType — çfarë vendos PD të bëjë në çdo kthesë
// ─────────────────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PdResponseType {
    Help,
    Orient,
    Clarify,
    ChallengeGentle,
    ChallengeWithFacts,
    ScaffoldBullet,
    ScaffoldMeasure,
    ScaffoldStructure,
    Listen,
}

impl PdResponseType {
    pub fn label(self) -> &'static str {
        match self {
            PdResponseType::Help => "HELP",
            PdResponseType::Orient => "ORIENT",
            PdResponseType::Clarify => "CLARIFY",
            PdResponseType::ChallengeGentle => "CHALLENGE_GENTLE",
            PdResponseType::ChallengeWithFacts => "CHALLENGE_WITH_FACTS",
            PdResponseType::ScaffoldBullet => "SCAFFOLD_BULLET",
            PdResponseType::ScaffoldMeasure => "SCAFFOLD_MEASURE",
            PdResponseType::ScaffoldStructure => "SCAFFOLD_STRUCTURE",
            PdResponseType::Listen => "LISTEN",
        }
    }

    /// is_challenge — a është kundërshtim (konsumon buxhet). Zero if — match.
    pub fn is_challenge(self) -> bool {
        match self {
            PdResponseType::ChallengeGentle | PdResponseType::ChallengeWithFacts => true,
            _ => false,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// PdTurn — çdo kthesë (sinjali + vendimi i PD)
// ─────────────────────────────────────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct PdTurn {
    pub turn_id: String,
    pub turn_index: u32,
    pub user_signal: CognitiveSignal,
    pub pd_response_type: PdResponseType,
    pub pd_response_hint: String,
    pub challenge_issued: bool,
    pub mode_at_turn: DebateMode,
    pub genius_check: Option<u32>,
}

// ─────────────────────────────────────────────────────────────────────────────
// PdEngineOutput — output i motorit pas çdo kthese
// ─────────────────────────────────────────────────────────────────────────────
#[derive(Debug, Clone)]
pub enum PdEngineOutput {
    Continue {
        turn: PdTurn,
    },
    GeniusDetected {
        genius_signal: GeniusSignal,
        package: PdCognitivePackage,
    },
    BudgetExhausted {
        trace_id: String,
    },
    SessionClosed {
        session_id: String,
        reason: String,
    },
}
#[derive(Debug, Clone)]
pub struct PdPreClassification {
    pub suggested_category: String, // "Hypothesis" | "PotentialHypothesis"
    pub confidence_hint: u32,       // fixed mass [3_000..7_000]
    pub reasoning: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// PdCognitivePackage — material final për PIM
// ─────────────────────────────────────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct PdCognitivePackage {
    pub package_id: String,
    pub session_id: String,
    pub genius_signal: GeniusSignal,
    pub pre_classification: PdPreClassification,
    pub accumulated_text: String, // konkatenim i sinjaleve kryesore
    pub dominant_concept: String,
    pub concept_territory: String,
    pub accumulated_mass: u32,
    pub structural_coherence: u32,
    pub accumulated_turns: u32,
    pub estimated_trl: u8, // TRL0/1/2 — vlerësim i PD
    pub origin: String,    // "PROGRESSIVE_DEBATIC"
    pub created_at: u64,
}

// ─────────────────────────────────────────────────────────────────────────────
// PdError
// ─────────────────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PdError {
    SessionNotFound(String),
}

impl std::fmt::Display for PdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PdError::SessionNotFound(m) => write!(f, "PD session not found: {m}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pd_id_is_deterministic_and_12_hex() {
        let a = pd_id(&[b"sess", b"trace"]);
        let b = pd_id(&[b"sess", b"trace"]);
        assert_eq!(a, b);
        assert_eq!(a.len(), 12);
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn budget_max_matches_mode() {
        assert_eq!(PressureBudget::max_for(DebateMode::Intellect), 2);
        assert_eq!(PressureBudget::max_for(DebateMode::Philosophy), 3);
        assert_eq!(PressureBudget::max_for(DebateMode::Scientific), 4);
        assert_eq!(PressureBudget::max_for(DebateMode::Novel), 2);
    }

    #[test]
    fn response_challenge_classification() {
        assert!(PdResponseType::ChallengeGentle.is_challenge());
        assert!(PdResponseType::ChallengeWithFacts.is_challenge());
        assert!(!PdResponseType::Help.is_challenge());
        assert!(!PdResponseType::Listen.is_challenge());
    }
}
