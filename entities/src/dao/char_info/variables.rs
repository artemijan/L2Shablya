
#[derive(Debug, Clone)]
pub enum CharVariables {
    VisualHairStyleId,
    VisualHairColorId,
    VisualFaceId,
    HairAccessoryEnabled,
    VitalityItemsUsed,
}

impl CharVariables {
    #[must_use]
    pub fn as_key(&self) -> &'static str {
        match self {
            CharVariables::VisualHairStyleId => "visualHairStyleId",
            CharVariables::VisualHairColorId => "visualHairColorId",
            CharVariables::VisualFaceId => "visualFaceId",
            CharVariables::HairAccessoryEnabled => "hairAccessoryEnabled",
            CharVariables::VitalityItemsUsed => "vitalityItemsUsed",
        }
    }
}
