/// Month and weekday names, AM/PM markers and default patterns of a locale,
/// used by [`DateTimeFormatter`](crate::DateTimeFormatter).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DateTimeLocale {
    /// Month names in the form used in dates (`%B`), from January.
    pub months: [&'static str; 12],
    /// Standalone month names (`%OB`), from January.
    pub months_standalone: [&'static str; 12],
    /// Abbreviated month names (`%b`), from January.
    pub months_abbreviated: [&'static str; 12],
    /// Weekday names (`%A`), from Sunday.
    pub weekdays: [&'static str; 7],
    /// Abbreviated weekday names (`%a`), from Sunday.
    pub weekdays_abbreviated: [&'static str; 7],
    /// Morning marker (`%p` before noon).
    pub am: &'static str,
    /// Afternoon marker (`%p` from noon).
    pub pm: &'static str,
    /// Date pattern (`%x`).
    pub date: &'static str,
    /// Time pattern (`%X`).
    pub time: &'static str,
}

const fn same_months(
    months: [&'static str; 12],
    months_abbreviated: [&'static str; 12],
    weekdays: [&'static str; 7],
    weekdays_abbreviated: [&'static str; 7],
    am_pm: [&'static str; 2],
    date: &'static str,
    time: &'static str,
) -> DateTimeLocale {
    DateTimeLocale {
        months,
        months_standalone: months,
        months_abbreviated,
        weekdays,
        weekdays_abbreviated,
        am: am_pm[0],
        pm: am_pm[1],
        date,
        time,
    }
}

const fn with_patterns(
    locale: DateTimeLocale,
    date: &'static str,
    time: &'static str,
) -> DateTimeLocale {
    DateTimeLocale {
        date,
        time,
        ..locale
    }
}

const TIME_24: &str = "%H:%M:%S";
const TIME_12: &str = "%I:%M:%S %p";

/// English (United States) names and patterns.
pub const ENGLISH: DateTimeLocale = same_months(
    [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ],
    [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ],
    [
        "Sunday",
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
    ],
    ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"],
    ["AM", "PM"],
    "%m/%d/%Y",
    TIME_12,
);

const FRENCH: DateTimeLocale = same_months(
    [
        "janvier",
        "février",
        "mars",
        "avril",
        "mai",
        "juin",
        "juillet",
        "août",
        "septembre",
        "octobre",
        "novembre",
        "décembre",
    ],
    [
        "janv.", "févr.", "mars", "avr.", "mai", "juin", "juil.", "août", "sept.", "oct.", "nov.",
        "déc.",
    ],
    [
        "dimanche", "lundi", "mardi", "mercredi", "jeudi", "vendredi", "samedi",
    ],
    ["dim.", "lun.", "mar.", "mer.", "jeu.", "ven.", "sam."],
    ["AM", "PM"],
    "%d/%m/%Y",
    TIME_24,
);

const GERMAN: DateTimeLocale = same_months(
    [
        "Januar",
        "Februar",
        "März",
        "April",
        "Mai",
        "Juni",
        "Juli",
        "August",
        "September",
        "Oktober",
        "November",
        "Dezember",
    ],
    [
        "Jan.", "Feb.", "März", "Apr.", "Mai", "Juni", "Juli", "Aug.", "Sept.", "Okt.", "Nov.",
        "Dez.",
    ],
    [
        "Sonntag",
        "Montag",
        "Dienstag",
        "Mittwoch",
        "Donnerstag",
        "Freitag",
        "Samstag",
    ],
    ["So.", "Mo.", "Di.", "Mi.", "Do.", "Fr.", "Sa."],
    ["AM", "PM"],
    "%d.%m.%Y",
    TIME_24,
);

const SPANISH: DateTimeLocale = same_months(
    [
        "enero",
        "febrero",
        "marzo",
        "abril",
        "mayo",
        "junio",
        "julio",
        "agosto",
        "septiembre",
        "octubre",
        "noviembre",
        "diciembre",
    ],
    [
        "ene", "feb", "mar", "abr", "may", "jun", "jul", "ago", "sept", "oct", "nov", "dic",
    ],
    [
        "domingo",
        "lunes",
        "martes",
        "miércoles",
        "jueves",
        "viernes",
        "sábado",
    ],
    ["dom", "lun", "mar", "mié", "jue", "vie", "sáb"],
    ["a. m.", "p. m."],
    "%d/%m/%Y",
    TIME_24,
);

const ITALIAN: DateTimeLocale = same_months(
    [
        "gennaio",
        "febbraio",
        "marzo",
        "aprile",
        "maggio",
        "giugno",
        "luglio",
        "agosto",
        "settembre",
        "ottobre",
        "novembre",
        "dicembre",
    ],
    [
        "gen", "feb", "mar", "apr", "mag", "giu", "lug", "ago", "set", "ott", "nov", "dic",
    ],
    [
        "domenica",
        "lunedì",
        "martedì",
        "mercoledì",
        "giovedì",
        "venerdì",
        "sabato",
    ],
    ["dom", "lun", "mar", "mer", "gio", "ven", "sab"],
    ["AM", "PM"],
    "%d/%m/%Y",
    TIME_24,
);

const PORTUGUESE: DateTimeLocale = same_months(
    [
        "janeiro",
        "fevereiro",
        "março",
        "abril",
        "maio",
        "junho",
        "julho",
        "agosto",
        "setembro",
        "outubro",
        "novembro",
        "dezembro",
    ],
    [
        "jan.", "fev.", "mar.", "abr.", "mai.", "jun.", "jul.", "ago.", "set.", "out.", "nov.",
        "dez.",
    ],
    [
        "domingo",
        "segunda-feira",
        "terça-feira",
        "quarta-feira",
        "quinta-feira",
        "sexta-feira",
        "sábado",
    ],
    ["dom.", "seg.", "ter.", "qua.", "qui.", "sex.", "sáb."],
    ["AM", "PM"],
    "%d/%m/%Y",
    TIME_24,
);

const DUTCH: DateTimeLocale = same_months(
    [
        "januari",
        "februari",
        "maart",
        "april",
        "mei",
        "juni",
        "juli",
        "augustus",
        "september",
        "oktober",
        "november",
        "december",
    ],
    [
        "jan", "feb", "mrt", "apr", "mei", "jun", "jul", "aug", "sep", "okt", "nov", "dec",
    ],
    [
        "zondag",
        "maandag",
        "dinsdag",
        "woensdag",
        "donderdag",
        "vrijdag",
        "zaterdag",
    ],
    ["zo", "ma", "di", "wo", "do", "vr", "za"],
    ["a.m.", "p.m."],
    "%d-%m-%Y",
    TIME_24,
);

const SWEDISH: DateTimeLocale = same_months(
    [
        "januari",
        "februari",
        "mars",
        "april",
        "maj",
        "juni",
        "juli",
        "augusti",
        "september",
        "oktober",
        "november",
        "december",
    ],
    [
        "jan.", "feb.", "mars", "apr.", "maj", "juni", "juli", "aug.", "sep.", "okt.", "nov.",
        "dec.",
    ],
    [
        "söndag", "måndag", "tisdag", "onsdag", "torsdag", "fredag", "lördag",
    ],
    ["sön", "mån", "tis", "ons", "tors", "fre", "lör"],
    ["fm", "em"],
    "%Y-%m-%d",
    TIME_24,
);

const NORWEGIAN: DateTimeLocale = same_months(
    [
        "januar",
        "februar",
        "mars",
        "april",
        "mai",
        "juni",
        "juli",
        "august",
        "september",
        "oktober",
        "november",
        "desember",
    ],
    [
        "jan.", "feb.", "mar.", "apr.", "mai", "jun.", "jul.", "aug.", "sep.", "okt.", "nov.",
        "des.",
    ],
    [
        "søndag", "mandag", "tirsdag", "onsdag", "torsdag", "fredag", "lørdag",
    ],
    ["søn.", "man.", "tir.", "ons.", "tor.", "fre.", "lør."],
    ["a.m.", "p.m."],
    "%d.%m.%Y",
    TIME_24,
);

const DANISH: DateTimeLocale = same_months(
    [
        "januar",
        "februar",
        "marts",
        "april",
        "maj",
        "juni",
        "juli",
        "august",
        "september",
        "oktober",
        "november",
        "december",
    ],
    [
        "jan.", "feb.", "mar.", "apr.", "maj", "jun.", "jul.", "aug.", "sep.", "okt.", "nov.",
        "dec.",
    ],
    [
        "søndag", "mandag", "tirsdag", "onsdag", "torsdag", "fredag", "lørdag",
    ],
    ["søn.", "man.", "tirs.", "ons.", "tors.", "fre.", "lør."],
    ["AM", "PM"],
    "%d.%m.%Y",
    TIME_24,
);

const POLISH: DateTimeLocale = DateTimeLocale {
    months: [
        "stycznia",
        "lutego",
        "marca",
        "kwietnia",
        "maja",
        "czerwca",
        "lipca",
        "sierpnia",
        "września",
        "października",
        "listopada",
        "grudnia",
    ],
    months_standalone: [
        "styczeń",
        "luty",
        "marzec",
        "kwiecień",
        "maj",
        "czerwiec",
        "lipiec",
        "sierpień",
        "wrzesień",
        "październik",
        "listopad",
        "grudzień",
    ],
    months_abbreviated: [
        "sty", "lut", "mar", "kwi", "maj", "cze", "lip", "sie", "wrz", "paź", "lis", "gru",
    ],
    weekdays: [
        "niedziela",
        "poniedziałek",
        "wtorek",
        "środa",
        "czwartek",
        "piątek",
        "sobota",
    ],
    weekdays_abbreviated: ["niedz.", "pon.", "wt.", "śr.", "czw.", "pt.", "sob."],
    am: "AM",
    pm: "PM",
    date: "%d.%m.%Y",
    time: TIME_24,
};

const CZECH: DateTimeLocale = DateTimeLocale {
    months: [
        "ledna",
        "února",
        "března",
        "dubna",
        "května",
        "června",
        "července",
        "srpna",
        "září",
        "října",
        "listopadu",
        "prosince",
    ],
    months_standalone: [
        "leden",
        "únor",
        "březen",
        "duben",
        "květen",
        "červen",
        "červenec",
        "srpen",
        "září",
        "říjen",
        "listopad",
        "prosinec",
    ],
    months_abbreviated: [
        "led", "úno", "bře", "dub", "kvě", "čvn", "čvc", "srp", "zář", "říj", "lis", "pro",
    ],
    weekdays: [
        "neděle",
        "pondělí",
        "úterý",
        "středa",
        "čtvrtek",
        "pátek",
        "sobota",
    ],
    weekdays_abbreviated: ["ne", "po", "út", "st", "čt", "pá", "so"],
    am: "dop.",
    pm: "odp.",
    date: "%d.%m.%Y",
    time: TIME_24,
};

const SLOVAK: DateTimeLocale = DateTimeLocale {
    months: [
        "januára",
        "februára",
        "marca",
        "apríla",
        "mája",
        "júna",
        "júla",
        "augusta",
        "septembra",
        "októbra",
        "novembra",
        "decembra",
    ],
    months_standalone: [
        "január",
        "február",
        "marec",
        "apríl",
        "máj",
        "jún",
        "júl",
        "august",
        "september",
        "október",
        "november",
        "december",
    ],
    months_abbreviated: [
        "jan", "feb", "mar", "apr", "máj", "jún", "júl", "aug", "sep", "okt", "nov", "dec",
    ],
    weekdays: [
        "nedeľa", "pondelok", "utorok", "streda", "štvrtok", "piatok", "sobota",
    ],
    weekdays_abbreviated: ["ne", "po", "ut", "st", "št", "pi", "so"],
    am: "AM",
    pm: "PM",
    date: "%d.%m.%Y",
    time: TIME_24,
};

const HUNGARIAN: DateTimeLocale = same_months(
    [
        "január",
        "február",
        "március",
        "április",
        "május",
        "június",
        "július",
        "augusztus",
        "szeptember",
        "október",
        "november",
        "december",
    ],
    [
        "jan.", "febr.", "márc.", "ápr.", "máj.", "jún.", "júl.", "aug.", "szept.", "okt.", "nov.",
        "dec.",
    ],
    [
        "vasárnap",
        "hétfő",
        "kedd",
        "szerda",
        "csütörtök",
        "péntek",
        "szombat",
    ],
    ["V", "H", "K", "Sze", "Cs", "P", "Szo"],
    ["de.", "du."],
    "%Y.%m.%d.",
    TIME_24,
);

const ROMANIAN: DateTimeLocale = same_months(
    [
        "ianuarie",
        "februarie",
        "martie",
        "aprilie",
        "mai",
        "iunie",
        "iulie",
        "august",
        "septembrie",
        "octombrie",
        "noiembrie",
        "decembrie",
    ],
    [
        "ian.", "feb.", "mar.", "apr.", "mai", "iun.", "iul.", "aug.", "sept.", "oct.", "nov.",
        "dec.",
    ],
    [
        "duminică",
        "luni",
        "marți",
        "miercuri",
        "joi",
        "vineri",
        "sâmbătă",
    ],
    ["dum.", "lun.", "mar.", "mie.", "joi", "vin.", "sâm."],
    ["a.m.", "p.m."],
    "%d.%m.%Y",
    TIME_24,
);

const TURKISH: DateTimeLocale = same_months(
    [
        "Ocak", "Şubat", "Mart", "Nisan", "Mayıs", "Haziran", "Temmuz", "Ağustos", "Eylül", "Ekim",
        "Kasım", "Aralık",
    ],
    [
        "Oca", "Şub", "Mar", "Nis", "May", "Haz", "Tem", "Ağu", "Eyl", "Eki", "Kas", "Ara",
    ],
    [
        "Pazar",
        "Pazartesi",
        "Salı",
        "Çarşamba",
        "Perşembe",
        "Cuma",
        "Cumartesi",
    ],
    ["Paz", "Pzt", "Sal", "Çar", "Per", "Cum", "Cmt"],
    ["ÖÖ", "ÖS"],
    "%d.%m.%Y",
    TIME_24,
);

const RUSSIAN: DateTimeLocale = DateTimeLocale {
    months: [
        "января",
        "февраля",
        "марта",
        "апреля",
        "мая",
        "июня",
        "июля",
        "августа",
        "сентября",
        "октября",
        "ноября",
        "декабря",
    ],
    months_standalone: [
        "январь",
        "февраль",
        "март",
        "апрель",
        "май",
        "июнь",
        "июль",
        "август",
        "сентябрь",
        "октябрь",
        "ноябрь",
        "декабрь",
    ],
    months_abbreviated: [
        "янв.",
        "февр.",
        "мар.",
        "апр.",
        "мая",
        "июн.",
        "июл.",
        "авг.",
        "сент.",
        "окт.",
        "нояб.",
        "дек.",
    ],
    weekdays: [
        "воскресенье",
        "понедельник",
        "вторник",
        "среда",
        "четверг",
        "пятница",
        "суббота",
    ],
    weekdays_abbreviated: ["вс", "пн", "вт", "ср", "чт", "пт", "сб"],
    am: "AM",
    pm: "PM",
    date: "%d.%m.%Y",
    time: TIME_24,
};

const UKRAINIAN: DateTimeLocale = DateTimeLocale {
    months: [
        "січня",
        "лютого",
        "березня",
        "квітня",
        "травня",
        "червня",
        "липня",
        "серпня",
        "вересня",
        "жовтня",
        "листопада",
        "грудня",
    ],
    months_standalone: [
        "січень",
        "лютий",
        "березень",
        "квітень",
        "травень",
        "червень",
        "липень",
        "серпень",
        "вересень",
        "жовтень",
        "листопад",
        "грудень",
    ],
    months_abbreviated: [
        "січ.",
        "лют.",
        "бер.",
        "квіт.",
        "трав.",
        "черв.",
        "лип.",
        "серп.",
        "вер.",
        "жовт.",
        "лист.",
        "груд.",
    ],
    weekdays: [
        "неділя",
        "понеділок",
        "вівторок",
        "середа",
        "четвер",
        "пʼятниця",
        "субота",
    ],
    weekdays_abbreviated: ["нд", "пн", "вт", "ср", "чт", "пт", "сб"],
    am: "дп",
    pm: "пп",
    date: "%d.%m.%Y",
    time: TIME_24,
};

const BELARUSIAN: DateTimeLocale = DateTimeLocale {
    months: [
        "студзеня",
        "лютага",
        "сакавіка",
        "красавіка",
        "мая",
        "чэрвеня",
        "ліпеня",
        "жніўня",
        "верасня",
        "кастрычніка",
        "лістапада",
        "снежня",
    ],
    months_standalone: [
        "студзень",
        "люты",
        "сакавік",
        "красавік",
        "май",
        "чэрвень",
        "ліпень",
        "жнівень",
        "верасень",
        "кастрычнік",
        "лістапад",
        "снежань",
    ],
    months_abbreviated: [
        "студ.",
        "лют.",
        "сак.",
        "крас.",
        "мая",
        "чэр.",
        "ліп.",
        "жн.",
        "вер.",
        "кастр.",
        "ліст.",
        "снеж.",
    ],
    weekdays: [
        "нядзеля",
        "панядзелак",
        "аўторак",
        "серада",
        "чацвер",
        "пятніца",
        "субота",
    ],
    weekdays_abbreviated: ["нд", "пн", "аў", "ср", "чц", "пт", "сб"],
    am: "AM",
    pm: "PM",
    date: "%d.%m.%Y",
    time: TIME_24,
};

const GREEK: DateTimeLocale = DateTimeLocale {
    months: [
        "Ιανουαρίου",
        "Φεβρουαρίου",
        "Μαρτίου",
        "Απριλίου",
        "Μαΐου",
        "Ιουνίου",
        "Ιουλίου",
        "Αυγούστου",
        "Σεπτεμβρίου",
        "Οκτωβρίου",
        "Νοεμβρίου",
        "Δεκεμβρίου",
    ],
    months_standalone: [
        "Ιανουάριος",
        "Φεβρουάριος",
        "Μάρτιος",
        "Απρίλιος",
        "Μάιος",
        "Ιούνιος",
        "Ιούλιος",
        "Αύγουστος",
        "Σεπτέμβριος",
        "Οκτώβριος",
        "Νοέμβριος",
        "Δεκέμβριος",
    ],
    months_abbreviated: [
        "Ιαν", "Φεβ", "Μαρ", "Απρ", "Μαΐ", "Ιουν", "Ιουλ", "Αυγ", "Σεπ", "Οκτ", "Νοε", "Δεκ",
    ],
    weekdays: [
        "Κυριακή",
        "Δευτέρα",
        "Τρίτη",
        "Τετάρτη",
        "Πέμπτη",
        "Παρασκευή",
        "Σάββατο",
    ],
    weekdays_abbreviated: ["Κυρ", "Δευ", "Τρί", "Τετ", "Πέμ", "Παρ", "Σάβ"],
    am: "π.μ.",
    pm: "μ.μ.",
    date: "%d/%m/%Y",
    time: TIME_12,
};

const ARABIC: DateTimeLocale = same_months(
    [
        "يناير",
        "فبراير",
        "مارس",
        "أبريل",
        "مايو",
        "يونيو",
        "يوليو",
        "أغسطس",
        "سبتمبر",
        "أكتوبر",
        "نوفمبر",
        "ديسمبر",
    ],
    [
        "يناير",
        "فبراير",
        "مارس",
        "أبريل",
        "مايو",
        "يونيو",
        "يوليو",
        "أغسطس",
        "سبتمبر",
        "أكتوبر",
        "نوفمبر",
        "ديسمبر",
    ],
    [
        "الأحد",
        "الاثنين",
        "الثلاثاء",
        "الأربعاء",
        "الخميس",
        "الجمعة",
        "السبت",
    ],
    [
        "الأحد",
        "الاثنين",
        "الثلاثاء",
        "الأربعاء",
        "الخميس",
        "الجمعة",
        "السبت",
    ],
    ["ص", "م"],
    "%d/%m/%Y",
    TIME_12,
);

const HEBREW: DateTimeLocale = same_months(
    [
        "ינואר",
        "פברואר",
        "מרץ",
        "אפריל",
        "מאי",
        "יוני",
        "יולי",
        "אוגוסט",
        "ספטמבר",
        "אוקטובר",
        "נובמבר",
        "דצמבר",
    ],
    [
        "ינו׳", "פבר׳", "מרץ", "אפר׳", "מאי", "יוני", "יולי", "אוג׳", "ספט׳", "אוק׳", "נוב׳",
        "דצמ׳",
    ],
    [
        "יום ראשון",
        "יום שני",
        "יום שלישי",
        "יום רביעי",
        "יום חמישי",
        "יום שישי",
        "יום שבת",
    ],
    [
        "יום א׳",
        "יום ב׳",
        "יום ג׳",
        "יום ד׳",
        "יום ה׳",
        "יום ו׳",
        "שבת",
    ],
    ["לפנה״צ", "אחה״צ"],
    "%d.%m.%Y",
    TIME_24,
);

const JAPANESE: DateTimeLocale = same_months(
    [
        "1月", "2月", "3月", "4月", "5月", "6月", "7月", "8月", "9月", "10月", "11月", "12月",
    ],
    [
        "1月", "2月", "3月", "4月", "5月", "6月", "7月", "8月", "9月", "10月", "11月", "12月",
    ],
    [
        "日曜日",
        "月曜日",
        "火曜日",
        "水曜日",
        "木曜日",
        "金曜日",
        "土曜日",
    ],
    ["日", "月", "火", "水", "木", "金", "土"],
    ["午前", "午後"],
    "%Y/%m/%d",
    TIME_24,
);

const CHINESE: DateTimeLocale = same_months(
    [
        "一月",
        "二月",
        "三月",
        "四月",
        "五月",
        "六月",
        "七月",
        "八月",
        "九月",
        "十月",
        "十一月",
        "十二月",
    ],
    [
        "1月", "2月", "3月", "4月", "5月", "6月", "7月", "8月", "9月", "10月", "11月", "12月",
    ],
    [
        "星期日",
        "星期一",
        "星期二",
        "星期三",
        "星期四",
        "星期五",
        "星期六",
    ],
    ["周日", "周一", "周二", "周三", "周四", "周五", "周六"],
    ["上午", "下午"],
    "%Y/%m/%d",
    TIME_24,
);

const KOREAN: DateTimeLocale = same_months(
    [
        "1월", "2월", "3월", "4월", "5월", "6월", "7월", "8월", "9월", "10월", "11월", "12월",
    ],
    [
        "1월", "2월", "3월", "4월", "5월", "6월", "7월", "8월", "9월", "10월", "11월", "12월",
    ],
    [
        "일요일",
        "월요일",
        "화요일",
        "수요일",
        "목요일",
        "금요일",
        "토요일",
    ],
    ["일", "월", "화", "수", "목", "금", "토"],
    ["오전", "오후"],
    "%Y. %m. %d.",
    "%p %I:%M:%S",
);

const HINDI: DateTimeLocale = same_months(
    [
        "जनवरी",
        "फ़रवरी",
        "मार्च",
        "अप्रैल",
        "मई",
        "जून",
        "जुलाई",
        "अगस्त",
        "सितंबर",
        "अक्तूबर",
        "नवंबर",
        "दिसंबर",
    ],
    [
        "जन॰",
        "फ़र॰",
        "मार्च",
        "अप्रैल",
        "मई",
        "जून",
        "जुल॰",
        "अग॰",
        "सित॰",
        "अक्तू॰",
        "नव॰",
        "दिस॰",
    ],
    [
        "रविवार",
        "सोमवार",
        "मंगलवार",
        "बुधवार",
        "गुरुवार",
        "शुक्रवार",
        "शनिवार",
    ],
    ["रवि", "सोम", "मंगल", "बुध", "गुरु", "शुक्र", "शनि"],
    ["am", "pm"],
    "%d/%m/%Y",
    TIME_12,
);

const THAI: DateTimeLocale = same_months(
    [
        "มกราคม",
        "กุมภาพันธ์",
        "มีนาคม",
        "เมษายน",
        "พฤษภาคม",
        "มิถุนายน",
        "กรกฎาคม",
        "สิงหาคม",
        "กันยายน",
        "ตุลาคม",
        "พฤศจิกายน",
        "ธันวาคม",
    ],
    [
        "ม.ค.",
        "ก.พ.",
        "มี.ค.",
        "เม.ย.",
        "พ.ค.",
        "มิ.ย.",
        "ก.ค.",
        "ส.ค.",
        "ก.ย.",
        "ต.ค.",
        "พ.ย.",
        "ธ.ค.",
    ],
    [
        "วันอาทิตย์",
        "วันจันทร์",
        "วันอังคาร",
        "วันพุธ",
        "วันพฤหัสบดี",
        "วันศุกร์",
        "วันเสาร์",
    ],
    ["อา.", "จ.", "อ.", "พ.", "พฤ.", "ศ.", "ส."],
    ["ก่อนเที่ยง", "หลังเที่ยง"],
    "%d/%m/%Y",
    TIME_24,
);

const VIETNAMESE: DateTimeLocale = same_months(
    [
        "tháng 1",
        "tháng 2",
        "tháng 3",
        "tháng 4",
        "tháng 5",
        "tháng 6",
        "tháng 7",
        "tháng 8",
        "tháng 9",
        "tháng 10",
        "tháng 11",
        "tháng 12",
    ],
    [
        "thg 1", "thg 2", "thg 3", "thg 4", "thg 5", "thg 6", "thg 7", "thg 8", "thg 9", "thg 10",
        "thg 11", "thg 12",
    ],
    [
        "Chủ Nhật",
        "Thứ Hai",
        "Thứ Ba",
        "Thứ Tư",
        "Thứ Năm",
        "Thứ Sáu",
        "Thứ Bảy",
    ],
    ["CN", "Th 2", "Th 3", "Th 4", "Th 5", "Th 6", "Th 7"],
    ["SA", "CH"],
    "%d/%m/%Y",
    TIME_24,
);

const fn subtag(locale: &str, wanted: usize) -> (usize, usize) {
    let bytes = locale.as_bytes();
    let mut index = 0;
    let mut start = 0;
    let mut current = 0;
    while index <= bytes.len() {
        if index == bytes.len() || bytes[index] == b'-' || bytes[index] == b'_' {
            if current == wanted {
                return (start, index);
            }
            current += 1;
            start = index + 1;
        }
        index += 1;
    }
    (0, 0)
}

const fn is(locale: &str, range: (usize, usize), candidate: &str) -> bool {
    let bytes = locale.as_bytes();
    let candidate = candidate.as_bytes();
    if range.1 - range.0 != candidate.len() {
        return false;
    }
    let mut index = 0;
    while index < candidate.len() {
        if !bytes[range.0 + index].eq_ignore_ascii_case(&candidate[index]) {
            return false;
        }
        index += 1;
    }
    true
}

/// Returns the built-in names and patterns of a locale, or `None` if the
/// language is not covered.
pub const fn find_date_time_locale(locale: &str) -> Option<DateTimeLocale> {
    let locale = locale.trim_ascii();
    let language = subtag(locale, 0);
    let region = subtag(locale, 1);
    Some(if is(locale, language, "en") {
        if is(locale, region, "GB")
            || is(locale, region, "IE")
            || is(locale, region, "AU")
            || is(locale, region, "NZ")
            || is(locale, region, "IN")
        {
            with_patterns(ENGLISH, "%d/%m/%Y", TIME_24)
        } else if is(locale, region, "CA") {
            with_patterns(ENGLISH, "%Y-%m-%d", TIME_12)
        } else {
            ENGLISH
        }
    } else if is(locale, language, "fr") {
        if is(locale, region, "CA") {
            with_patterns(FRENCH, "%Y-%m-%d", TIME_24)
        } else if is(locale, region, "CH") {
            with_patterns(FRENCH, "%d.%m.%Y", TIME_24)
        } else {
            FRENCH
        }
    } else if is(locale, language, "de") {
        GERMAN
    } else if is(locale, language, "es") {
        SPANISH
    } else if is(locale, language, "it") {
        ITALIAN
    } else if is(locale, language, "pt") {
        PORTUGUESE
    } else if is(locale, language, "nl") {
        DUTCH
    } else if is(locale, language, "sv") {
        SWEDISH
    } else if is(locale, language, "no") || is(locale, language, "nb") || is(locale, language, "nn")
    {
        NORWEGIAN
    } else if is(locale, language, "da") {
        DANISH
    } else if is(locale, language, "pl") {
        POLISH
    } else if is(locale, language, "cs") {
        CZECH
    } else if is(locale, language, "sk") {
        SLOVAK
    } else if is(locale, language, "hu") {
        HUNGARIAN
    } else if is(locale, language, "ro") {
        ROMANIAN
    } else if is(locale, language, "tr") {
        TURKISH
    } else if is(locale, language, "ru") {
        RUSSIAN
    } else if is(locale, language, "uk") {
        UKRAINIAN
    } else if is(locale, language, "be") {
        BELARUSIAN
    } else if is(locale, language, "el") {
        GREEK
    } else if is(locale, language, "ar") {
        ARABIC
    } else if is(locale, language, "he") || is(locale, language, "iw") {
        HEBREW
    } else if is(locale, language, "ja") {
        JAPANESE
    } else if is(locale, language, "zh") {
        CHINESE
    } else if is(locale, language, "ko") {
        KOREAN
    } else if is(locale, language, "hi") {
        HINDI
    } else if is(locale, language, "th") {
        THAI
    } else if is(locale, language, "vi") {
        VIETNAMESE
    } else {
        return None;
    })
}

/// Returns the built-in names and patterns of a locale, English if the language
/// is not covered.
///
/// Called in `const` context, only the selected locale is linked; called with
/// a runtime string, the whole table is.
///
/// ```
/// use embedded_i18n::date_time_locale;
///
/// const FRENCH: embedded_i18n::DateTimeLocale = date_time_locale("fr-CA");
/// assert_eq!(FRENCH.months[7], "août");
/// assert_eq!(FRENCH.date, "%Y-%m-%d");
/// ```
pub const fn date_time_locale(locale: &str) -> DateTimeLocale {
    match find_date_time_locale(locale) {
        Some(found) => found,
        None => ENGLISH,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_locales_by_language() {
        assert_eq!(date_time_locale("fr").months[7], "août");
        assert_eq!(date_time_locale("FR_ch").date, "%d.%m.%Y");
        assert_eq!(date_time_locale("fr-CA").date, "%Y-%m-%d");
        assert_eq!(date_time_locale("de-AT").weekdays[0], "Sonntag");
        assert_eq!(date_time_locale("nb").months[11], "desember");
        assert_eq!(date_time_locale("zh-Hant-TW").am, "上午");
    }

    #[test]
    fn english_variants() {
        assert_eq!(date_time_locale("en").date, "%m/%d/%Y");
        assert_eq!(date_time_locale("en-US").time, "%I:%M:%S %p");
        assert_eq!(date_time_locale("en-GB").date, "%d/%m/%Y");
        assert_eq!(date_time_locale("en_gb").time, "%H:%M:%S");
    }

    #[test]
    fn unknown_locales_fall_back_to_english() {
        assert!(find_date_time_locale("tlh").is_none());
        assert!(find_date_time_locale("").is_none());
        assert_eq!(date_time_locale("tlh"), ENGLISH);
    }

    #[test]
    fn genitive_and_standalone_months() {
        let russian = date_time_locale("ru");
        assert_eq!(russian.months[9], "октября");
        assert_eq!(russian.months_standalone[9], "октябрь");
        assert_eq!(date_time_locale("fr").months_standalone[1], "février");
    }
}
