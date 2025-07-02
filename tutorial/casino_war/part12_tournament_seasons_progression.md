# Casino War Tutorial - Part 12: Tournament Seasons & Progression - Building a Living Game

## What We're Building in Part 12

Creating a persistent, evolving tournament ecosystem:

1. **Season System**: 4-week competitive seasons with unique themes and rewards
2. **Player Progression**: XP, levels, and skill-based matchmaking
3. **Unlockable Content**: New card backs, table themes, AI personalities, and features
4. **Achievement System**: Challenges that reward mastery and exploration
5. **Persistent Profiles**: Save system for long-term player investment

## Understanding Game Progression Systems

### The Player Retention Problem

Imagine you're designing a game that players return to for months or years. We need systems that:
- Provide clear short-term and long-term goals
- Reward both skill improvement and time investment
- Create a sense of meaningful progression
- Offer variety to prevent staleness
- Build player investment through customization

Let's think about this like building a competitive sport league:
1. **Seasons**: Regular resets with fresh competition
2. **Rankings**: Clear skill tiers and progression paths
3. **Rewards**: Trophies, medals, and recognition
4. **Personal Growth**: Skill development over time
5. **Community**: Shared experiences and competition

In programming terms, this is a **persistent progression system** combined with **content management** and **reward scheduling**. We need to create compelling reasons for players to keep coming back.

## Section 1: Season System Architecture

First, let's design our seasonal tournament structure. Think of this like organizing a year-long sports league:

```rust
// Season configuration and state
#[derive(Resource, Serialize, Deserialize)]
struct SeasonManager {
    current_season: Season,
    season_history: Vec<CompletedSeason>,
    next_season_start: SystemTime,
    season_duration: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Season {
    id: u32,
    name: String,
    theme: SeasonTheme,
    start_date: SystemTime,
    end_date: SystemTime,
    
    // Special rules/modifiers for this season
    modifiers: Vec<SeasonModifier>,
    
    // Rewards
    reward_track: RewardTrack,
    exclusive_unlocks: Vec<UnlockableContent>,
    
    // Leaderboards
    global_leaderboard: Vec<LeaderboardEntry>,
    regional_leaderboards: HashMap<Region, Vec<LeaderboardEntry>>,
    
    // Statistics
    total_players: u32,
    total_tournaments: u32,
    total_hands_played: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum SeasonTheme {
    // Each season has unique visual theme and gameplay twist
    HighStakes {         // Double or nothing emphasis
        chip_multiplier: f32,
        war_bonus: f32,
    },
    SpeedDemon {         // Faster tournaments, quick decisions
        time_limit: f32,
        decision_bonus: f32,
    },
    Underdog {          // Bonuses for comeback victories
        comeback_multiplier: f32,
        upset_bonus: u32,
    },
    MasterStrategist {   // Rewards perfect play
        perfect_hand_bonus: u32,
        strategy_score_weight: f32,
    },
    ChaosCarnival {      // Random events and wild modifiers
        event_frequency: f32,
        randomness_factor: f32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum SeasonModifier {
    // Gameplay modifications active during season
    DoubleBets,              // All bets worth 2x
    NoSurrender,             // Must war on ties
    TurboMode,               // 1-minute tournaments
    EliminationBonus(u32),   // Bonus for eliminating opponents
    StreakRewards,           // Increasing rewards for win streaks
    RandomEvents,            // Chaos events during play
    AllInFridays,            // Special rules on Fridays
}

// Reward progression system
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RewardTrack {
    tiers: Vec<RewardTier>,
    current_progress: u32,
    claimed_rewards: HashSet<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RewardTier {
    tier_number: u32,
    required_xp: u32,
    rewards: Vec<Reward>,
    is_premium: bool,  // Premium battle pass style
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Reward {
    Currency { amount: u32, currency_type: CurrencyType },
    UnlockableContent(UnlockableContent),
    XpBoost { multiplier: f32, duration: Duration },
    Title(String),
    Emote(String),
    ProfileBorder(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum CurrencyType {
    Chips,           // Basic currency
    Gems,            // Premium currency
    SeasonTokens,    // Season-specific
}
```

**Season System Design:**

**4-Week Cycles:**
- Fresh start maintains competitive balance
- Prevents runaway leaders
- Creates anticipation for new content
- Matches typical player attention spans

**Themed Seasons:**
Each season has unique personality:
- **HighStakes**: Risk/reward focused
- **SpeedDemon**: Fast-paced action
- **Underdog**: Comeback mechanics
- **MasterStrategist**: Skill emphasis
- **ChaosCarnival**: Unpredictable fun

**Season Modifiers:**
- Change core gameplay temporarily
- Create variety without complexity
- Test new mechanics safely
- Keep veterans engaged

```rust
impl SeasonManager {
    fn start_new_season(&mut self) -> Result<(), SeasonError> {
        // Archive current season
        if let Some(completed) = self.complete_current_season() {
            self.season_history.push(completed);
        }
        
        // Generate next season
        let season_id = self.season_history.len() as u32 + 1;
        let theme = self.select_next_theme();
        
        self.current_season = Season {
            id: season_id,
            name: format!("Season {} - {}", season_id, theme.name()),
            theme,
            start_date: SystemTime::now(),
            end_date: SystemTime::now() + self.season_duration,
            modifiers: theme.generate_modifiers(),
            reward_track: RewardTrack::new(season_id),
            exclusive_unlocks: generate_season_unlocks(season_id, &theme),
            global_leaderboard: Vec::new(),
            regional_leaderboards: HashMap::new(),
            total_players: 0,
            total_tournaments: 0,
            total_hands_played: 0,
        };
        
        // Schedule next season
        self.next_season_start = self.current_season.end_date;
        
        Ok(())
    }
    
    fn select_next_theme(&self) -> SeasonTheme {
        // Rotate through themes, avoiding recent repeats
        let recent_themes: Vec<_> = self.season_history.iter()
            .rev()
            .take(3)
            .map(|s| &s.theme)
            .collect();
            
        // Select theme that hasn't been used recently
        let available_themes = vec![
            SeasonTheme::HighStakes { chip_multiplier: 2.0, war_bonus: 1.5 },
            SeasonTheme::SpeedDemon { time_limit: 60.0, decision_bonus: 1.2 },
            SeasonTheme::Underdog { comeback_multiplier: 2.0, upset_bonus: 500 },
            SeasonTheme::MasterStrategist { perfect_hand_bonus: 100, strategy_score_weight: 1.5 },
            SeasonTheme::ChaosCarnival { event_frequency: 0.3, randomness_factor: 2.0 },
        ];
        
        available_themes.into_iter()
            .find(|theme| !recent_themes.iter().any(|&t| 
                std::mem::discriminant(t) == std::mem::discriminant(theme)
            ))
            .unwrap_or(SeasonTheme::HighStakes { chip_multiplier: 2.0, war_bonus: 1.5 })
    }
}

// Season-specific gameplay modifications
fn apply_season_modifiers(
    season: &Season,
    mut game_rules: ResMut<GameRules>,
    mut scoring: ResMut<ScoringSystem>,
) {
    // Reset to base rules
    *game_rules = GameRules::default();
    *scoring = ScoringSystem::default();
    
    // Apply theme modifications
    match &season.theme {
        SeasonTheme::HighStakes { chip_multiplier, war_bonus } => {
            game_rules.base_bet_multiplier = *chip_multiplier;
            game_rules.war_payout_multiplier = *war_bonus;
        },
        SeasonTheme::SpeedDemon { time_limit, decision_bonus } => {
            game_rules.tournament_time_limit = *time_limit;
            scoring.quick_decision_bonus = *decision_bonus;
        },
        SeasonTheme::Underdog { comeback_multiplier, upset_bonus } => {
            scoring.comeback_multiplier = *comeback_multiplier;
            scoring.upset_victory_bonus = *upset_bonus;
        },
        SeasonTheme::MasterStrategist { perfect_hand_bonus, strategy_score_weight } => {
            scoring.perfect_play_bonus = *perfect_hand_bonus;
            scoring.strategy_weight = *strategy_score_weight;
        },
        SeasonTheme::ChaosCarnival { event_frequency, randomness_factor } => {
            game_rules.random_event_chance = *event_frequency;
            game_rules.chaos_factor = *randomness_factor;
        },
    }
    
    // Apply additional modifiers
    for modifier in &season.modifiers {
        match modifier {
            SeasonModifier::DoubleBets => {
                game_rules.base_bet_multiplier *= 2.0;
            },
            SeasonModifier::NoSurrender => {
                game_rules.war_is_mandatory = true;
            },
            SeasonModifier::TurboMode => {
                game_rules.tournament_time_limit = 60.0;
            },
            SeasonModifier::EliminationBonus(bonus) => {
                scoring.elimination_bonus = *bonus;
            },
            _ => {}
        }
    }
}
```

**Theme Implementation:**
- Each theme modifies specific game rules
- Themes rotate to maintain freshness
- Previous 3 themes excluded from selection
- Graceful fallback if all themes recent

**Modifier System:**
- Modifiers stack with theme rules
- Can be combined for unique experiences
- Easy to add new modifiers
- Clear impact on gameplay

## Section 2: Player Progression System

Now let's create the persistent player progression that spans seasons:

```rust
// Player profile with persistent progression
#[derive(Component, Serialize, Deserialize)]
struct PlayerProfile {
    // Identity
    player_id: Uuid,
    username: String,
    created_at: SystemTime,
    
    // Progression
    level: u32,
    total_xp: u64,
    current_season_xp: u32,
    prestige_level: u32,  // Resets at max level
    
    // Statistics
    lifetime_stats: LifetimeStatistics,
    season_stats: HashMap<u32, SeasonStatistics>,
    
    // Unlocks and customization
    unlocked_content: HashSet<UnlockableContent>,
    equipped_items: EquippedItems,
    
    // Currency
    currencies: HashMap<CurrencyType, u32>,
    
    // Achievements
    achievements: HashMap<AchievementId, AchievementProgress>,
    completed_achievements: HashSet<AchievementId>,
    
    // Skill rating
    skill_rating: SkillRating,
    matchmaking_rank: MatchmakingRank,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LifetimeStatistics {
    total_tournaments: u32,
    tournament_wins: u32,
    total_hands_played: u64,
    total_hands_won: u64,
    total_chips_won: u64,
    total_chips_lost: u64,
    highest_chip_count: u32,
    longest_win_streak: u32,
    perfect_tournaments: u32,  // Won without losing a hand
    
    // Advanced stats
    average_tournament_placement: f32,
    win_rate_by_position: HashMap<usize, f32>,
    favorite_ai_opponent: Option<AiPersonality>,
    nemesis_ai_opponent: Option<AiPersonality>,
}

// XP and leveling system
#[derive(Debug, Clone)]
struct LevelingSystem {
    xp_curve: XpCurve,
    max_level: u32,
    prestige_enabled: bool,
    level_rewards: HashMap<u32, Vec<Reward>>,
}

#[derive(Debug, Clone)]
enum XpCurve {
    Linear { base: u32, increment: u32 },
    Exponential { base: f32, exponent: f32 },
    Custom(Vec<u32>),  // XP required for each level
}

impl LevelingSystem {
    fn calculate_xp_for_level(&self, level: u32) -> u64 {
        match &self.xp_curve {
            XpCurve::Linear { base, increment } => {
                (*base + (*increment * (level - 1))) as u64
            },
            XpCurve::Exponential { base, exponent } => {
                (base * (level as f32).powf(*exponent)) as u64
            },
            XpCurve::Custom(thresholds) => {
                thresholds.get(level as usize - 1)
                    .copied()
                    .unwrap_or(u32::MAX) as u64
            },
        }
    }
    
    fn calculate_level_from_xp(&self, total_xp: u64) -> u32 {
        let mut level = 1;
        let mut required_xp = 0u64;
        
        while level < self.max_level {
            required_xp += self.calculate_xp_for_level(level);
            if total_xp < required_xp {
                break;
            }
            level += 1;
        }
        
        level
    }
    
    fn award_xp(&mut self, profile: &mut PlayerProfile, amount: u32, source: XpSource) -> Vec<LevelUpReward> {
        let old_level = profile.level;
        
        profile.total_xp += amount as u64;
        profile.current_season_xp += amount;
        
        let new_level = self.calculate_level_from_xp(profile.total_xp);
        profile.level = new_level;
        
        // Check for level ups
        let mut rewards = Vec::new();
        for level in (old_level + 1)..=new_level {
            if let Some(level_rewards) = self.level_rewards.get(&level) {
                rewards.push(LevelUpReward {
                    level,
                    rewards: level_rewards.clone(),
                });
            }
            
            // Check for prestige
            if level == self.max_level && self.prestige_enabled {
                profile.prestige_level += 1;
                profile.level = 1;
                profile.total_xp = 0;
                
                rewards.push(LevelUpReward {
                    level: 0,  // Special prestige level
                    rewards: vec![
                        Reward::Title(format!("Prestige {}", profile.prestige_level)),
                        Reward::ProfileBorder(format!("prestige_{}", profile.prestige_level)),
                    ],
                });
            }
        }
        
        info!("Awarded {} XP from {:?}. Level {} -> {}", 
              amount, source, old_level, new_level);
        
        rewards
    }
}

#[derive(Debug, Clone)]
enum XpSource {
    TournamentComplete { placement: usize },
    HandWon { was_war: bool },
    AchievementComplete { achievement_id: AchievementId },
    DailyBonus,
    SeasonReward,
    SpecialEvent,
}

// Skill-based matchmaking
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SkillRating {
    rating: f32,           // Elo-style rating
    confidence: f32,       // How certain we are of rating
    games_played: u32,
    peak_rating: f32,
    current_streak: i32,  // Positive = wins, negative = losses
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum MatchmakingRank {
    Bronze,
    Silver, 
    Gold,
    Platinum,
    Diamond,
    Master,
    GrandMaster,
}

impl SkillRating {
    fn update_after_match(&mut self, opponents: &[(f32, bool)], placement: usize) {
        // Simplified Elo calculation
        let k_factor = if self.games_played < 30 { 40.0 } else { 20.0 };
        
        let expected_score: f32 = opponents.iter()
            .map(|(opponent_rating, _)| {
                1.0 / (1.0 + 10.0_f32.powf((opponent_rating - self.rating) / 400.0))
            })
            .sum::<f32>() / opponents.len() as f32;
            
        let actual_score = 1.0 - (placement as f32 / (opponents.len() + 1) as f32);
        
        self.rating += k_factor * (actual_score - expected_score);
        self.games_played += 1;
        
        // Update confidence
        self.confidence = (self.games_played as f32 / 100.0).min(1.0);
        
        // Update peak
        if self.rating > self.peak_rating {
            self.peak_rating = self.rating;
        }
        
        // Update streak
        if placement == 1 {
            self.current_streak = self.current_streak.max(0) + 1;
        } else {
            self.current_streak = self.current_streak.min(0) - 1;
        }
    }
    
    fn get_rank(&self) -> MatchmakingRank {
        match self.rating as u32 {
            0..=999 => MatchmakingRank::Bronze,
            1000..=1299 => MatchmakingRank::Silver,
            1300..=1599 => MatchmakingRank::Gold,
            1600..=1899 => MatchmakingRank::Platinum,
            1900..=2199 => MatchmakingRank::Diamond,
            2200..=2499 => MatchmakingRank::Master,
            _ => MatchmakingRank::GrandMaster,
        }
    }
}
```

**Player Profile Architecture:**
- **UUID**: Globally unique player identification
- **Multi-tier progression**: Levels, prestige, and skill rating
- **Comprehensive statistics**: Track everything for analytics
- **Currency management**: Multiple currency types
- **Achievement tracking**: Progress and completion

**XP System Design:**
- **Flexible curves**: Linear, exponential, or custom
- **Level rewards**: Automatic unlocks at milestones
- **Prestige system**: Reset for hardcore players
- **Multiple XP sources**: Varied ways to progress

**Skill Rating (Elo) System:**
```rust
1.0 / (1.0 + 10.0_f32.powf((opponent_rating - self.rating) / 400.0))
```
- Standard Elo formula for expected performance
- K-factor higher for new players (faster adjustment)
- Confidence increases with games played
- Separate from XP progression

## Section 3: Unlockable Content System

Let's create the content that players can unlock and customize:

```rust
// Unlockable content types
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
enum UnlockableContent {
    CardBack(CardBackStyle),
    TableTheme(TableTheme),
    ChipStyle(ChipStyle),
    CardEffects(CardEffectSet),
    AiPersonality(CustomAiPersonality),
    Emote(EmoteId),
    Title(TitleId),
    ProfileBorder(BorderId),
    VictoryAnimation(AnimationId),
    Music(MusicTrackId),
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
struct CardBackStyle {
    id: String,
    name: String,
    rarity: Rarity,
    animated: bool,
    particle_effects: Option<ParticleEffectId>,
    unlock_requirement: UnlockRequirement,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
struct TableTheme {
    id: String,
    name: String,
    description: String,
    
    // Visual components
    felt_color: Color,
    felt_texture: Option<TextureId>,
    edge_style: EdgeStyle,
    lighting_preset: LightingPreset,
    
    // Special effects
    ambient_particles: Option<ParticleEffectId>,
    win_effects: Option<SpecialEffectId>,
    
    // Audio
    ambient_sound: Option<AudioId>,
    card_sounds: CardSoundSet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
    Mythic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum UnlockRequirement {
    Level(u32),
    Achievement(AchievementId),
    SeasonRank { season: u32, rank: u32 },
    TournamentWins(u32),
    PerfectGames(u32),
    Currency { currency_type: CurrencyType, amount: u32 },
    SpecialEvent(String),
    SecretCondition(SecretUnlockId),
}

// Customization system
#[derive(Component, Serialize, Deserialize)]
struct EquippedItems {
    card_back: CardBackStyle,
    table_theme: TableTheme,
    chip_style: ChipStyle,
    card_effects: Option<CardEffectSet>,
    
    // Profile customization
    title: Option<TitleId>,
    border: Option<BorderId>,
    
    // Gameplay customization
    victory_animation: AnimationId,
    emotes: [Option<EmoteId>; 8],  // Quick emote wheel
    
    // Audio
    music_playlist: Vec<MusicTrackId>,
    sound_pack: SoundPackId,
}

// Content generation for new unlocks
fn generate_season_unlocks(season_id: u32, theme: &SeasonTheme) -> Vec<UnlockableContent> {
    let mut unlocks = Vec::new();
    
    // Season-exclusive card back
    unlocks.push(UnlockableContent::CardBack(CardBackStyle {
        id: format!("season_{}_exclusive", season_id),
        name: format!("{} Champion", theme.name()),
        rarity: Rarity::Legendary,
        animated: true,
        particle_effects: Some(ParticleEffectId(format!("season_{}_particles", season_id))),
        unlock_requirement: UnlockRequirement::SeasonRank { 
            season: season_id, 
            rank: 100  // Top 100 players
        },
    }));
    
    // Theme-specific table
    unlocks.push(UnlockableContent::TableTheme(generate_themed_table(season_id, theme)));
    
    // Milestone rewards
    for milestone in [10, 25, 50, 75, 100] {
        unlocks.push(generate_milestone_reward(season_id, milestone));
    }
    
    unlocks
}

// Dynamic content loading system
#[derive(Resource)]
struct ContentManager {
    loaded_content: HashMap<UnlockableContent, LoadedAsset>,
    content_catalog: ContentCatalog,
    download_queue: VecDeque<ContentDownload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ContentCatalog {
    version: u32,
    items: HashMap<String, ContentMetadata>,
    bundles: HashMap<String, ContentBundle>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ContentMetadata {
    id: String,
    content_type: ContentType,
    file_size: u64,
    download_url: String,
    dependencies: Vec<String>,
    preview_url: Option<String>,
}

impl ContentManager {
    fn ensure_content_available(&mut self, content: &UnlockableContent) -> Result<(), ContentError> {
        if self.loaded_content.contains_key(content) {
            return Ok(());
        }
        
        // Queue for download if not available
        let content_id = content.get_id();
        if let Some(metadata) = self.content_catalog.items.get(&content_id) {
            self.download_queue.push_back(ContentDownload {
                content: content.clone(),
                metadata: metadata.clone(),
                priority: DownloadPriority::Normal,
                retry_count: 0,
            });
        }
        
        Err(ContentError::NotAvailable)
    }
}
```

**Content System Design:**

**Multiple Content Types:**
- **Visual**: Card backs, tables, effects
- **Audio**: Music, sound packs
- **Profile**: Titles, borders, emotes
- **Gameplay**: AI personalities, animations

**Rarity Tiers:**
- Creates collection desire
- Visual feedback for achievement
- Supports loot box mechanics (if desired)
- Clear progression goals

**Dynamic Content Loading:**
- Content catalog for discovery
- On-demand downloading
- Dependency management
- Preview before unlock

## Section 4: Achievement System

Let's create a comprehensive achievement system that drives engagement:

```rust
// Achievement definitions
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
struct AchievementId(String);

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Achievement {
    id: AchievementId,
    name: String,
    description: String,
    category: AchievementCategory,
    
    // Requirements
    criteria: AchievementCriteria,
    hidden: bool,  // Secret achievements
    
    // Rewards
    xp_reward: u32,
    currency_rewards: Vec<(CurrencyType, u32)>,
    unlock_rewards: Vec<UnlockableContent>,
    
    // Display
    icon: IconId,
    rarity: Rarity,
    display_priority: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum AchievementCategory {
    Tournament,      // Tournament victories and placements
    Combat,          // Hand wins, wars, perfect games
    Collection,      // Unlocking content
    Social,          // Playing with friends, spectating
    Mastery,         // High-skill achievements
    Exploration,     // Trying different modes/features
    Seasonal,        // Season-specific achievements
    Hidden,          // Secret achievements
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum AchievementCriteria {
    // Simple counters
    WinTournaments(u32),
    WinHands(u32),
    WinWars(u32),
    PlayGames(u32),
    
    // Conditional achievements
    WinWithoutLosing,  // Perfect tournament
    WinFromBehind { deficit: u32 },  // Comeback victory
    WinInTime { seconds: u32 },  // Speed achievement
    
    // Collection
    UnlockItems { count: u32, rarity: Option<Rarity> },
    CollectFullSet { set_name: String },
    
    // Complex criteria
    Composite(Vec<AchievementCriteria>),  // All must be met
    Alternative(Vec<AchievementCriteria>), // Any can be met
    
    // Progressive
    Progressive {
        stages: Vec<(u32, String)>,  // (requirement, description)
        current_stage: usize,
    },
}

// Achievement tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AchievementProgress {
    achievement_id: AchievementId,
    started_at: SystemTime,
    current_progress: AchievementProgressData,
    completed: bool,
    completed_at: Option<SystemTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum AchievementProgressData {
    Counter { current: u32, target: u32 },
    Boolean(bool),
    Stages { current_stage: usize, stage_progress: Box<AchievementProgressData> },
    Composite(Vec<AchievementProgressData>),
}

// Achievement manager
#[derive(Resource)]
struct AchievementManager {
    definitions: HashMap<AchievementId, Achievement>,
    trigger_map: HashMap<AchievementTrigger, Vec<AchievementId>>,
    notification_queue: VecDeque<AchievementNotification>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
enum AchievementTrigger {
    TournamentEnd,
    HandComplete,
    WarComplete,
    LevelUp,
    UnlockItem,
    SeasonEnd,
    Custom(String),
}

impl AchievementManager {
    fn check_achievements(
        &mut self,
        trigger: AchievementTrigger,
        context: &AchievementContext,
        profile: &mut PlayerProfile,
    ) -> Vec<CompletedAchievement> {
        let mut completed = Vec::new();
        
        if let Some(achievement_ids) = self.trigger_map.get(&trigger) {
            for achievement_id in achievement_ids {
                if let Some(achievement) = self.definitions.get(achievement_id) {
                    // Skip if already completed
                    if profile.completed_achievements.contains(achievement_id) {
                        continue;
                    }
                    
                    // Get or create progress
                    let progress = profile.achievements
                        .entry(achievement_id.clone())
                        .or_insert_with(|| AchievementProgress {
                            achievement_id: achievement_id.clone(),
                            started_at: SystemTime::now(),
                            current_progress: self.create_initial_progress(&achievement.criteria),
                            completed: false,
                            completed_at: None,
                        });
                    
                    // Update progress
                    if self.update_progress(&achievement.criteria, &mut progress.current_progress, context) {
                        // Achievement completed!
                        progress.completed = true;
                        progress.completed_at = Some(SystemTime::now());
                        profile.completed_achievements.insert(achievement_id.clone());
                        
                        completed.push(CompletedAchievement {
                            achievement: achievement.clone(),
                            completed_at: SystemTime::now(),
                        });
                        
                        // Queue notification
                        self.notification_queue.push_back(AchievementNotification {
                            achievement: achievement.clone(),
                            unlock_time: SystemTime::now(),
                        });
                    }
                }
            }
        }
        
        completed
    }
    
    fn update_progress(
        &self,
        criteria: &AchievementCriteria,
        progress: &mut AchievementProgressData,
        context: &AchievementContext,
    ) -> bool {
        match (criteria, progress) {
            (AchievementCriteria::WinTournaments(target), AchievementProgressData::Counter { current, .. }) => {
                if context.action == AchievementAction::TournamentWin {
                    *current += 1;
                    *current >= *target
                } else {
                    false
                }
            },
            
            (AchievementCriteria::WinFromBehind { deficit }, AchievementProgressData::Boolean(_)) => {
                if let AchievementAction::TournamentWin = context.action {
                    if let Some(comeback_size) = context.comeback_from {
                        if comeback_size >= *deficit {
                            *progress = AchievementProgressData::Boolean(true);
                            return true;
                        }
                    }
                }
                false
            },
            
            (AchievementCriteria::Progressive { stages, .. }, AchievementProgressData::Stages { current_stage, stage_progress }) => {
                if *current_stage >= stages.len() {
                    return true;  // All stages complete
                }
                
                // Check current stage progress
                if let Some((requirement, _)) = stages.get(*current_stage) {
                    if let AchievementProgressData::Counter { current, .. } = stage_progress.as_mut() {
                        *current += 1;
                        if current >= requirement {
                            *current_stage += 1;
                            if *current_stage < stages.len() {
                                // Reset progress for next stage
                                *stage_progress = Box::new(AchievementProgressData::Counter { 
                                    current: 0, 
                                    target: stages[*current_stage].0 
                                });
                            }
                        }
                    }
                }
                
                *current_stage >= stages.len()
            },
            
            _ => false,
        }
    }
}

// Generate achievements for the game
fn create_all_achievements() -> Vec<Achievement> {
    vec![
        // Tournament achievements
        Achievement {
            id: AchievementId("first_win".to_string()),
            name: "First Victory".to_string(),
            description: "Win your first tournament".to_string(),
            category: AchievementCategory::Tournament,
            criteria: AchievementCriteria::WinTournaments(1),
            hidden: false,
            xp_reward: 100,
            currency_rewards: vec![(CurrencyType::Chips, 500)],
            unlock_rewards: vec![],
            icon: IconId("trophy_bronze".to_string()),
            rarity: Rarity::Common,
            display_priority: 100,
        },
        
        // Progressive achievement
        Achievement {
            id: AchievementId("tournament_master".to_string()),
            name: "Tournament Master".to_string(),
            description: "Master the art of tournament play".to_string(),
            category: AchievementCategory::Mastery,
            criteria: AchievementCriteria::Progressive {
                stages: vec![
                    (10, "Win 10 tournaments".to_string()),
                    (50, "Win 50 tournaments".to_string()),
                    (100, "Win 100 tournaments".to_string()),
                    (500, "Win 500 tournaments".to_string()),
                ],
                current_stage: 0,
            },
            hidden: false,
            xp_reward: 1000,
            currency_rewards: vec![(CurrencyType::Gems, 100)],
            unlock_rewards: vec![
                UnlockableContent::Title(TitleId("Tournament Master".to_string())),
            ],
            icon: IconId("trophy_platinum".to_string()),
            rarity: Rarity::Legendary,
            display_priority: 10,
        },
        
        // Hidden achievement
        Achievement {
            id: AchievementId("perfect_chaos".to_string()),
            name: "Perfect Chaos".to_string(),
            description: "???".to_string(),  // Hidden until unlocked
            category: AchievementCategory::Hidden,
            criteria: AchievementCriteria::Composite(vec![
                AchievementCriteria::WinTournaments(1),
                AchievementCriteria::WinWithoutLosing,
            ]),
            hidden: true,
            xp_reward: 500,
            currency_rewards: vec![],
            unlock_rewards: vec![
                UnlockableContent::CardBack(CardBackStyle {
                    id: "chaos_back".to_string(),
                    name: "Chaos Master".to_string(),
                    rarity: Rarity::Mythic,
                    animated: true,
                    particle_effects: Some(ParticleEffectId("chaos_particles".to_string())),
                    unlock_requirement: UnlockRequirement::Achievement(
                        AchievementId("perfect_chaos".to_string())
                    ),
                }),
            ],
            icon: IconId("chaos_icon".to_string()),
            rarity: Rarity::Mythic,
            display_priority: 1,
        },
    ]
}
```

**Achievement System Features:**

**Multiple Criteria Types:**
- Simple counters (win X games)
- Conditional (win from behind)
- Progressive (multi-stage)
- Composite (multiple requirements)

**Hidden Achievements:**
- Mystery descriptions until unlocked
- Encourages experimentation
- Special rewards for discovery

**Smart Tracking:**
- Trigger-based checking
- Efficient progress updates
- Persistent across sessions
- Queue notifications

## Section 5: Save System and Cloud Sync

Let's implement a robust save system with cloud synchronization:

```rust
// Save game management
#[derive(Resource)]
struct SaveManager {
    local_saves: HashMap<Uuid, LocalSave>,
    cloud_sync: Option<CloudSyncService>,
    autosave_timer: Timer,
    save_queue: VecDeque<SaveRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LocalSave {
    save_id: Uuid,
    profile: PlayerProfile,
    settings: GameSettings,
    timestamp: SystemTime,
    version: SaveVersion,
    checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SaveVersion {
    major: u32,
    minor: u32,
    patch: u32,
}

impl SaveManager {
    fn save_game(&mut self, profile: &PlayerProfile, settings: &GameSettings) -> Result<(), SaveError> {
        let save = LocalSave {
            save_id: Uuid::new_v4(),
            profile: profile.clone(),
            settings: settings.clone(),
            timestamp: SystemTime::now(),
            version: SaveVersion { major: 1, minor: 0, patch: 0 },
            checksum: self.calculate_checksum(profile, settings),
        };
        
        // Save locally
        self.save_to_disk(&save)?;
        
        // Queue for cloud sync
        if self.cloud_sync.is_some() {
            self.save_queue.push_back(SaveRequest {
                save: save.clone(),
                priority: SavePriority::Normal,
                retry_count: 0,
            });
        }
        
        self.local_saves.insert(profile.player_id, save);
        
        Ok(())
    }
    
    fn save_to_disk(&self, save: &LocalSave) -> Result<(), SaveError> {
        let save_path = self.get_save_path(&save.profile.player_id);
        
        // Create backup of existing save
        if save_path.exists() {
            let backup_path = save_path.with_extension("bak");
            std::fs::copy(&save_path, backup_path)?;
        }
        
        // Serialize with compression
        let data = bincode::serialize(save)?;
        let compressed = lz4::compress(&data);
        
        // Write atomically
        let temp_path = save_path.with_extension("tmp");
        std::fs::write(&temp_path, compressed)?;
        std::fs::rename(temp_path, save_path)?;
        
        Ok(())
    }
    
    fn load_game(&mut self, player_id: &Uuid) -> Result<PlayerProfile, SaveError> {
        // Try cloud first if available
        if let Some(cloud) = &mut self.cloud_sync {
            if let Ok(cloud_save) = cloud.fetch_save(player_id) {
                // Verify and merge with local if needed
                return self.merge_saves(cloud_save);
            }
        }
        
        // Fall back to local
        if let Some(save) = self.local_saves.get(player_id) {
            return Ok(save.profile.clone());
        }
        
        // Load from disk
        let save_path = self.get_save_path(player_id);
        if save_path.exists() {
            let compressed = std::fs::read(save_path)?;
            let data = lz4::decompress(&compressed)?;
            let save: LocalSave = bincode::deserialize(&data)?;
            
            // Verify checksum
            if save.checksum != self.calculate_checksum(&save.profile, &GameSettings::default()) {
                return Err(SaveError::ChecksumMismatch);
            }
            
            // Handle version migration if needed
            let migrated_save = self.migrate_save(save)?;
            
            self.local_saves.insert(*player_id, migrated_save.clone());
            
            Ok(migrated_save.profile)
        } else {
            Err(SaveError::NotFound)
        }
    }
    
    fn migrate_save(&self, mut save: LocalSave) -> Result<LocalSave, SaveError> {
        let current_version = SaveVersion { major: 1, minor: 0, patch: 0 };
        
        // Apply migrations based on version differences
        if save.version.major < current_version.major {
            // Major version migrations
            save = self.migrate_v0_to_v1(save)?;
        }
        
        save.version = current_version;
        Ok(save)
    }
}

// Cloud synchronization
struct CloudSyncService {
    api_client: ApiClient,
    sync_status: SyncStatus,
    conflict_resolution: ConflictResolution,
}

#[derive(Debug, Clone)]
enum SyncStatus {
    Synced,
    Syncing { progress: f32 },
    Conflict { local: LocalSave, cloud: LocalSave },
    Error(String),
}

#[derive(Debug, Clone)]
enum ConflictResolution {
    PreferNewer,      // Use save with latest timestamp
    PreferHigher,     // Use save with higher progress
    Manual,           // Ask user
}

impl CloudSyncService {
    async fn sync_saves(&mut self, local_saves: &HashMap<Uuid, LocalSave>) -> Result<(), SyncError> {
        self.sync_status = SyncStatus::Syncing { progress: 0.0 };
        
        for (player_id, local_save) in local_saves {
            // Fetch cloud version
            match self.api_client.get_save(player_id).await {
                Ok(cloud_save) => {
                    // Compare saves
                    if local_save.timestamp > cloud_save.timestamp {
                        // Upload local
                        self.api_client.upload_save(local_save).await?;
                    } else if local_save.timestamp < cloud_save.timestamp {
                        // Download cloud
                        // Would update local here
                    } else if local_save.checksum != cloud_save.checksum {
                        // Conflict!
                        self.sync_status = SyncStatus::Conflict {
                            local: local_save.clone(),
                            cloud: cloud_save,
                        };
                        
                        // Resolve based on strategy
                        match self.conflict_resolution {
                            ConflictResolution::PreferNewer => {
                                // Already handled above
                            },
                            ConflictResolution::PreferHigher => {
                                if local_save.profile.total_xp > cloud_save.profile.total_xp {
                                    self.api_client.upload_save(local_save).await?;
                                }
                            },
                            ConflictResolution::Manual => {
                                // Would trigger UI for user choice
                                return Err(SyncError::ManualResolutionRequired);
                            },
                        }
                    }
                },
                Err(_) => {
                    // No cloud save, upload local
                    self.api_client.upload_save(local_save).await?;
                },
            }
            
            self.sync_status = SyncStatus::Syncing { 
                progress: 1.0 / local_saves.len() as f32 
            };
        }
        
        self.sync_status = SyncStatus::Synced;
        Ok(())
    }
}
```

**Save System Features:**

**Local Storage:**
- Binary serialization with bincode
- LZ4 compression for space efficiency
- Atomic writes prevent corruption
- Automatic backups before overwrite

**Cloud Synchronization:**
- Async upload/download
- Conflict detection and resolution
- Progress tracking for UI
- Offline mode fallback

**Data Integrity:**
- Checksum verification
- Version migration system
- Corruption recovery from backups
- Save compatibility across versions

## Testing Your Progression System

At this point, you should be able to:

1. **Start New Season**: See themed tournament with special rules
2. **Earn XP and Level**: Watch progress bar fill and level up
3. **Unlock Content**: Earn new card backs and customizations
4. **Complete Achievements**: Get notifications for accomplishments
5. **Save Progress**: Persistent profile across sessions

## Key Concepts Mastered

1. **Season Design**: Creating variety through temporal content
   - Rotating themes and modifiers
   - Exclusive time-limited rewards
   - Fresh competition cycles

2. **Progression Psychology**: Understanding player motivation
   - Multiple progression axes (XP, skill, collection)
   - Short and long-term goals
   - Meaningful choices in customization

3. **Content Management**: Scalable unlockable system
   - Dynamic content loading
   - Rarity and collection mechanics
   - Preview and discovery systems

4. **Achievement Design**: Driving player behavior
   - Varied criteria types
   - Hidden discoveries
   - Progressive challenges

5. **Data Persistence**: Robust save systems
   - Local and cloud storage
   - Conflict resolution
   - Version migration

## Exercises

1. **Create Seasonal Events**: Add limited-time events within seasons

2. **Implement Leaderboard Rewards**: End-of-season prizes based on ranking

3. **Add Social Features**: Friend lists and gift giving

4. **Create Meta-Progression**: Account-wide unlocks across seasons

5. **Design Daily Challenges**: Rotating objectives for regular players

## Next Steps

In Part 13, we'll add:
- Spectator mode for watching live tournaments
- Tournament replay system
- Learning from recorded games
- Commentary and analysis tools

The progression system creates the foundation for a living game that evolves with its community!