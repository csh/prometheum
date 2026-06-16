/// Parsed [`NSLOCTEXT`](https://dev.epicgames.com/documentation/unreal-engine/text-localization-in-unreal-engine) 
pub struct LocalizableString {
    pub namespace: String,

    pub key: String,

    pub source: String,
}

/// Parsed [`FNameProperty`](https://dev.epicgames.com/documentation/unreal-engine/API/Runtime/CoreUObject/FNameProperty)
pub struct FNameProperty {
    pub key: String,
    pub value: String,
}