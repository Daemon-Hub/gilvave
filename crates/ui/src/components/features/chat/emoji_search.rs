//! Emoji categorization and multi-language search (Russian to English keyword mapper).

pub const EMOJI_GROUPS: &[(&str, &str, &str, emojis::Group)] = &[
    (
        "smileys",
        "😀",
        "Смайлики и эмоции",
        emojis::Group::SmileysAndEmotion,
    ),
    ("people", "👋", "Люди и жесты", emojis::Group::PeopleAndBody),
    (
        "animals",
        "🐱",
        "Животные и природа",
        emojis::Group::AnimalsAndNature,
    ),
    ("food", "🍔", "Еда и напитки", emojis::Group::FoodAndDrink),
    (
        "travel",
        "✈️",
        "Путешествия и места",
        emojis::Group::TravelAndPlaces,
    ),
    (
        "activities",
        "⚽",
        "Активности и спорт",
        emojis::Group::Activities,
    ),
    ("objects", "💡", "Предметы", emojis::Group::Objects),
    ("symbols", "❤️", "Символы", emojis::Group::Symbols),
    ("flags", "🚩", "Флаги", emojis::Group::Flags),
];

pub fn map_ru_query_to_en(ru: &str) -> &'static [&'static str] {
    let lower = ru.trim().to_lowercase();
    match lower.as_str() {
        s if s.starts_with("улыб")
            || s.starts_with("смех")
            || s.starts_with("радос")
            || s == "хаха"
            || s == "весел" =>
        {
            &["smile", "grin", "laugh", "happy", "joy"]
        }
        s if s.starts_with("смущ") || s.starts_with("мил") => &["blush", "shy"],
        s if s.starts_with("люб") || s.starts_with("серд") => &["heart", "love"],
        s if s.starts_with("поцел") => &["kiss"],
        s if s.starts_with("глаз") => &["eye", "eyes"],
        s if s.starts_with("слез") || s.starts_with("плач") || s.starts_with("рыд") => {
            &["cry", "sob", "tear"]
        }
        s if s.starts_with("груст") || s.starts_with("печал") || s.starts_with("тоск") => {
            &["sad", "pensive", "disappointed"]
        }
        s if s.starts_with("зло") || s.starts_with("гнев") || s.starts_with("ярос") => {
            &["angry", "rage", "mad"]
        }
        s if s.starts_with("шок") || s.starts_with("ужас") || s.starts_with("страх") => {
            &["scream", "fear", "shock"]
        }
        s if s.starts_with("сон") || s.starts_with("спат") || s.starts_with("устал") => {
            &["sleep", "yawn", "tired", "zzz"]
        }
        s if s.starts_with("дум") || s.starts_with("мысл") => &["thinking", "thought"],
        s if s.starts_with("крут") || s.starts_with("очки") => &["cool", "sunglasses"],
        s if s.starts_with("праздн") || s.starts_with("вечерин") || s.starts_with("ура") => {
            &["party", "tada", "celebration"]
        }
        s if s.starts_with("ботан") || s.starts_with("умн") => &["nerd"],
        s if s.starts_with("клоун") => &["clown"],
        s if s.starts_with("призрак") || s.starts_with("привид") => &["ghost"],
        s if s.starts_with("череп") || s.starts_with("смерт") => &["skull"],
        s if s.starts_with("огонь")
            || s.starts_with("плам")
            || s.starts_with("жар")
            || s == "топ" =>
        {
            &["fire", "flame"]
        }
        s if s.starts_with("звезд") || s.starts_with("блеск") || s.starts_with("сиян") => {
            &["star", "sparkle"]
        }
        s if s.starts_with("молни") || s.starts_with("гром") || s.starts_with("ток") => {
            &["zap", "lightning"]
        }
        s if s.starts_with("взрыв") || s.starts_with("бум") => {
            &["boom", "explosion", "collision"]
        }

        s if s.starts_with("рук")
            || s.starts_with("ладон")
            || s.starts_with("привет")
            || s.starts_with("пок") =>
        {
            &["hand", "wave", "raised"]
        }
        s if s.starts_with("лайк") || s.starts_with("класс") || s.starts_with("супер") => {
            &["thumbs up"]
        }
        s if s.starts_with("дизлайк") || s.starts_with("плох") => &["thumbs down"],
        s if s.starts_with("палец") || s.starts_with("пальц") => &["finger", "point"],
        s if s.starts_with("кулак") => &["fist"],
        s if s.starts_with("хлоп") || s.starts_with("аплодис") => &["clap"],
        s if s.starts_with("молитв") || s.starts_with("спасиб") || s.starts_with("пожалуйст") => {
            &["pray"]
        }
        s if s.starts_with("сил") || s.starts_with("бицепс") || s.starts_with("мышц") => {
            &["muscle"]
        }
        s if s.starts_with("ок") => &["ok"],
        s if s.starts_with("мир") || s.starts_with("побед") => &["peace", "victory"],

        s if s.starts_with("кот") || s.starts_with("кош") => &["cat"],
        s if s.starts_with("собак") || s.starts_with("пес") || s.starts_with("щен") => {
            &["dog"]
        }
        s if s.starts_with("лис") => &["fox"],
        s if s.starts_with("медвед") || s.starts_with("мишк") => &["bear"],
        s if s.starts_with("панд") => &["panda"],
        s if s.starts_with("лев") => &["lion"],
        s if s.starts_with("тигр") => &["tiger"],
        s if s.starts_with("обезьян") => &["monkey"],
        s if s.starts_with("лягушк") || s.starts_with("жаб") => &["frog"],
        s if s.starts_with("птиц") => &["bird"],
        s if s.starts_with("пингвин") => &["penguin"],
        s if s.starts_with("сов") => &["owl"],
        s if s.starts_with("бабочк") => &["butterfly"],
        s if s.starts_with("рыб") => &["fish"],
        s if s.starts_with("кит") => &["whale"],
        s if s.starts_with("краб") => &["crab"],
        s if s.starts_with("осьминог") => &["octopus"],

        s if s.starts_with("ед") || s.starts_with("кушат") => &["food"],
        s if s.starts_with("пицц") => &["pizza"],
        s if s.starts_with("бургер") => &["burger", "hamburger"],
        s if s.starts_with("картошк") || s.starts_with("фри") => &["fries"],
        s if s.starts_with("хотдог") => &["hotdog"],
        s if s.starts_with("торт") || s.starts_with("пирож") => &["cake"],
        s if s.starts_with("пончик") => &["donut"],
        s if s.starts_with("шоколад") => &["chocolate"],
        s if s.starts_with("печень") => &["cookie"],
        s if s.starts_with("кофе") => &["coffee"],
        s if s.starts_with("чай") => &["tea"],
        s if s.starts_with("пив") => &["beer"],
        s if s.starts_with("вин") || s.starts_with("бокал") => &["wine", "cheers"],
        s if s.starts_with("яблок") => &["apple"],
        s if s.starts_with("банан") => &["banana"],
        s if s.starts_with("клубник") => &["strawberry"],
        s if s.starts_with("арбуз") => &["watermelon"],

        s if s.starts_with("машин") || s.starts_with("авто") => &["car", "automobile"],
        s if s.starts_with("самолет") => &["airplane"],
        s if s.starts_with("поезд") => &["train"],
        s if s.starts_with("ракет") => &["rocket"],
        s if s.starts_with("дом") => &["house", "building"],
        s if s.starts_with("комп") || s.starts_with("ноут") => &["laptop", "computer"],
        s if s.starts_with("телефон") => &["phone", "mobile"],
        s if s.starts_with("наушник") => &["headphone"],
        s if s.starts_with("музык") || s.starts_with("нот") => &["music", "note"],
        s if s.starts_with("игр") || s.starts_with("гейм") => &["game", "video game"],
        s if s.starts_with("деньг") || s.starts_with("рубл") || s.starts_with("доллар") => {
            &["money", "dollar"]
        }
        s if s.starts_with("подар") => &["gift", "present"],
        s if s.starts_with("замок") => &["lock"],
        s if s.starts_with("ключ") => &["key"],
        s if s.starts_with("галочк") => &["check"],
        s if s.starts_with("крестик") => &["cross"],
        s if s.starts_with("флаг") => &["flag"],
        s if s.starts_with("росси") => &["russia"],
        _ => &[],
    }
}

pub fn matches_emoji_query(emoji: &emojis::Emoji, query: &str) -> bool {
    if emoji.as_str().contains(query) {
        return true;
    }
    let name_lower = emoji.name().to_lowercase();
    if name_lower.contains(query) {
        return true;
    }
    let sc_lower = emoji.shortcode().map(|sc| sc.to_lowercase());
    if let Some(ref sc) = sc_lower
        && sc.contains(query)
    {
        return true;
    }
    let en_keywords = map_ru_query_to_en(query);
    for kw in en_keywords {
        if name_lower.contains(kw) {
            return true;
        }
        if let Some(ref sc) = sc_lower
            && sc.contains(kw)
        {
            return true;
        }
    }
    false
}
