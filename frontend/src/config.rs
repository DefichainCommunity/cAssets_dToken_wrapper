use enum_table::{EnumTable, Enumable};

#[derive(Enumable, Copy, Clone)]
#[repr(u8)]
pub enum ConfigEntry{
    Native,
    WrappedNativeAddress,
    CAssetDTokenWrapRouter,
    CAssetDTokenWrapFactory,
    VanillaV2Router,
    VanillaV3Router,
}

static MAINNET: EnumTable<ConfigEntry, &'static str, { ConfigEntry::COUNT }> =
    enum_table::et!(ConfigEntry, &'static str, |t| match t {
        ConfigEntry::Native => "DFI",
        ConfigEntry::WrappedNativeAddress => "0x49febbF9626B2D39aBa11C01d83Ef59b3D56d2A4",
        ConfigEntry::CAssetDTokenWrapRouter => "",
        ConfigEntry::CAssetDTokenWrapFactory => "",
        ConfigEntry::VanillaV2Router => "0x3E8C92491fc73390166BA00725B8F5BD734B8fba",
        ConfigEntry::VanillaV3Router => "0x2A9c4EdE9994911359af815367187947eD1dDf02",
    });

static TESTNET: EnumTable<ConfigEntry, &'static str, { ConfigEntry::COUNT }> =
    enum_table::et!(ConfigEntry, &'static str, |t| match t {
        ConfigEntry::Native => "DFI",
        ConfigEntry::WrappedNativeAddress => "0x62AF40e6d8714eF9210AeF7e94A151c27673d7A9",
        ConfigEntry::CAssetDTokenWrapRouter => "0x7081cbaDb76F0df8eeB9889EFC821aFE6a451622",
        ConfigEntry::CAssetDTokenWrapFactory => "0xE521e9e0d066e7ba3702833E7B535Be6DE2fa41b",
        ConfigEntry::VanillaV2Router => "0x79208eADd9FbC29116108433a38Af62D0fD83850",
        ConfigEntry::VanillaV3Router => "",
    });

pub fn get_config_entry(chain_id: u32, config_entry: &ConfigEntry) -> &'static str {
    if chain_id == 1130{
        return MAINNET.get(config_entry);
    }else if chain_id == 1131{
        return TESTNET.get(config_entry);
    }
    ""
}
