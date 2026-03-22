use stardom_core::company::OfficeTier;
use stardom_core::game::GamePhase;
use stardom_core::stats::RecognitionTier;
use stardom_core::types::Activity;

pub fn phase_text(phase: GamePhase) -> &'static str {
    match phase {
        GamePhase::MainGame => "進行中",
        GamePhase::PostEnding => "延長賽",
        GamePhase::GameOver => "遊戲結束",
    }
}

pub fn office_tier_text(tier: OfficeTier) -> &'static str {
    match tier {
        OfficeTier::Starter => "簡陋",
        OfficeTier::Standard => "標準",
        OfficeTier::Premium => "高級",
        OfficeTier::Luxury => "豪華",
    }
}

pub fn recognition_tier_text(tier: RecognitionTier) -> &'static str {
    match tier {
        RecognitionTier::Unknown => "無名",
        RecognitionTier::Newcomer => "新人",
        RecognitionTier::Rising => "崛起",
        RecognitionTier::Established => "成名",
        RecognitionTier::Star => "明星",
        RecognitionTier::Superstar => "巨星",
    }
}

pub fn activity_text(activity: &Activity) -> &'static str {
    match activity {
        Activity::Training => "訓練中",
        Activity::PartTimeJob => "打工中",
        Activity::Gig => "通告中",
        Activity::Rest => "休息中",
        Activity::Idle => "待命",
    }
}
