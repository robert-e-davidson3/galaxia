use bevy_rapier2d::prelude::*;

pub const PLAYER_GROUP: Group = Group::GROUP_1;
pub const ETHER_GROUP: Group = Group::GROUP_2; // mostly LooseResource
pub const MINIGAME_CONTENTS_GROUP: Group = Group::GROUP_3; // stuff inside of minigames
pub const MINIGAME_AURA_GROUP: Group = Group::GROUP_4; // ether-minigame interaction
pub const BORDER_GROUP: Group = Group::GROUP_32; // borders around minigames

#[inline]
pub fn player_filter() -> Group {
    PLAYER_GROUP | ETHER_GROUP | BORDER_GROUP
}

#[inline]
pub fn ether_filter() -> Group {
    ETHER_GROUP | PLAYER_GROUP | MINIGAME_AURA_GROUP | BORDER_GROUP
}

#[inline]
pub fn minigame_contents_filter() -> Group {
    MINIGAME_CONTENTS_GROUP | BORDER_GROUP
}

#[inline]
pub fn minigame_aura_filter() -> Group {
    ETHER_GROUP
}

#[inline]
pub fn border_filter() -> Group {
    // !MINIGAME_AURA_GROUP
    BORDER_GROUP | PLAYER_GROUP | ETHER_GROUP | MINIGAME_CONTENTS_GROUP
}
