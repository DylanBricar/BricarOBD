/// Helper to get bilingual NRC description
fn nrc_description(lang: &str, fr: &str, en: &str) -> String {
    if lang == "fr" {
        fr.to_string()
    } else {
        en.to_string()
    }
}

/// Parse UDS negative response code into human-readable string
pub fn parse_negative_response(response: &str, lang: &str) -> String {
    let bytes: Vec<u8> = response
        .split_whitespace()
        .filter_map(|s| u8::from_str_radix(s, 16).ok())
        .collect();

    if bytes.len() >= 3 && bytes[0] == 0x7F {
        let nrc = bytes[2];
        match nrc {
            0x10 => nrc_description(lang, "Rejet général", "General reject"),
            0x11 => nrc_description(lang, "Service non supporté", "Service not supported"),
            0x12 => nrc_description(
                lang,
                "Sous-fonction non supportée",
                "Sub-function not supported",
            ),
            0x13 => nrc_description(
                lang,
                "Longueur de message invalide",
                "Invalid message length",
            ),
            0x14 => nrc_description(lang, "Réponse trop longue", "Response too long"),
            0x22 => nrc_description(lang, "Conditions non remplies", "Conditions not correct"),
            0x24 => nrc_description(
                lang,
                "Erreur de séquence de requête",
                "Request sequence error",
            ),
            0x25 => nrc_description(
                lang,
                "Pas de réponse du sous-réseau",
                "No response from sub-net",
            ),
            0x26 => nrc_description(
                lang,
                "Échec empêchant l'exécution",
                "Failure prevents execution",
            ),
            0x31 => nrc_description(lang, "Requête hors limites", "Request out of range"),
            0x33 => nrc_description(lang, "Accès sécurité refusé", "Security access denied"),
            0x35 => nrc_description(lang, "Clé invalide", "Invalid key"),
            0x36 => nrc_description(
                lang,
                "Nombre de tentatives dépassé",
                "Exceeded number of attempts",
            ),
            0x37 => nrc_description(
                lang,
                "Délai requis non écoulé",
                "Required time delay not expired",
            ),
            0x70 => nrc_description(
                lang,
                "Upload/download non accepté",
                "Upload/download not accepted",
            ),
            0x71 => nrc_description(
                lang,
                "Transfert de données suspendu",
                "Transfer data suspended",
            ),
            0x72 => nrc_description(
                lang,
                "Échec de programmation général",
                "General programming failure",
            ),
            0x73 => nrc_description(
                lang,
                "Compteur de séquence de bloc incorrect",
                "Wrong block sequence counter",
            ),
            0x78 => nrc_description(
                lang,
                "Réponse en attente (traitement en cours)",
                "Response pending (still processing)",
            ),
            0x7E => nrc_description(
                lang,
                "Sous-fonction non supportée dans la session active",
                "Sub-function not supported in active session",
            ),
            0x7F => nrc_description(
                lang,
                "Service non supporté dans la session active",
                "Service not supported in active session",
            ),
            _ => format!("NRC 0x{:02X}", nrc),
        }
    } else {
        response.to_string()
    }
}
