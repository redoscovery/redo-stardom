use std::collections::HashMap;

use stardom_core::gig::GigDef;
use stardom_core::job::JobDef;
use stardom_core::training::TrainingDef;

use bevy::prelude::*;

#[derive(Clone, Debug)]
pub enum PlannedActivity {
    Training(TrainingDef),
    Job(JobDef),
    Gig(GigDef),
    Rest,
}

impl PlannedActivity {
    pub fn label(&self) -> String {
        match self {
            PlannedActivity::Training(t) => format!("訓練：{}", t.name),
            PlannedActivity::Job(j) => format!("打工：{}", j.name),
            PlannedActivity::Gig(g) => format!("通告：{}", g.name),
            PlannedActivity::Rest => "休息".to_string(),
        }
    }
}

#[derive(Resource, Default)]
pub struct WeekPlan {
    pub assignments: HashMap<usize, PlannedActivity>,
}

impl WeekPlan {
    pub fn assign(&mut self, artist_index: usize, activity: PlannedActivity) {
        self.assignments.insert(artist_index, activity);
    }

    pub fn cancel(&mut self, artist_index: usize) {
        self.assignments.remove(&artist_index);
    }

    pub fn get(&self, artist_index: usize) -> Option<&PlannedActivity> {
        self.assignments.get(&artist_index)
    }

    /// Check whether all non-locked artists have been assigned an activity.
    pub fn all_assigned(&self, non_locked_indices: &[usize]) -> bool {
        !non_locked_indices.is_empty()
            && non_locked_indices
                .iter()
                .all(|idx| self.assignments.contains_key(idx))
    }

    pub fn clear(&mut self) {
        self.assignments.clear();
    }
}
