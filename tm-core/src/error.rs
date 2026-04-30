pub enum BuildingError {
    MissingInitialState,
    MissingInitialTape,
    MissingMoveType,
    MissingTapeSize,
    MissingAcceptingStates,
    MissingTransitions,
    InvalidTransition,
}
