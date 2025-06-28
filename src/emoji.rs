
use gtk4::prelude::*;
use gtk4::{Dialog, ResponseType, Box, Orientation, Label, Entry};
use crate::{editor, localization};

/// Show emoji picker dialog with search and categories
pub fn show_emoji_picker_dialog(editor: &editor::MarkdownEditor) {
    let dialog = Dialog::with_buttons(
        Some(&localization::tr("emoji.title")),
        None::<&gtk4::Window>,
        gtk4::DialogFlags::MODAL,
        &[("Close", ResponseType::Close)],
    );
    
    dialog.set_default_size(550, 600);
    dialog.set_resizable(true); // Allow resizing
    
    // Create main container with minimal margins
    let main_box = Box::new(Orientation::Vertical, 4); // Reduced spacing
    main_box.set_margin_top(6); // Reduced margin
    main_box.set_margin_bottom(6);
    main_box.set_margin_start(6);
    main_box.set_margin_end(6);
    
    // Search entry
    let search_box = Box::new(Orientation::Horizontal, 8);
    let search_label = Label::new(Some("Search:"));
    let search_entry = Entry::new();
    search_entry.set_placeholder_text(Some("Type to search emojis..."));
    search_entry.set_hexpand(true);
    
    search_box.append(&search_label);
    search_box.append(&search_entry);
    main_box.append(&search_box);
    
    // Categories - simplified with just emoji icons
    let categories_box = Box::new(Orientation::Horizontal, 2);
    let categories = vec![
        ("All", "all", "Show all emojis"),
        ("😀", "faces", "Face and emotion emojis"),
        ("❤️", "hearts", "Heart and love emojis"), 
        ("👋", "people", "People and body emojis"),
        ("🐶", "animals", "Animal emojis"),
        ("🌱", "nature", "Nature emojis"),
        ("🍎", "food", "Food and drink emojis"),
        ("⚽", "sports", "Sports and activities emojis"),
        ("🚗", "travel", "Travel and places emojis"),
        ("💡", "objects", "Objects emojis"),
        ("🔢", "symbols", "Symbols emojis"),
    ];
    
    let mut category_buttons = Vec::new();
    for (label, category, tooltip) in &categories {
        let button = gtk4::ToggleButton::with_label(label);
        button.set_css_classes(&["category-button"]);
        button.set_tooltip_text(Some(tooltip));
        if category == &"all" {
            button.set_active(true);
        }
        categories_box.append(&button);
        category_buttons.push((button, category.to_string()));
    }
    
    let categories_scroll = gtk4::ScrolledWindow::new();
    categories_scroll.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Never);
    categories_scroll.set_child(Some(&categories_box));
    categories_scroll.set_max_content_height(35); // Reduced height
    categories_scroll.set_vexpand(false);
    categories_scroll.set_margin_bottom(2); // Reduced margin
    main_box.append(&categories_scroll);
    
    // Emoji grid in scrolled window with minimal margins
    let emoji_scroll = gtk4::ScrolledWindow::new();
    emoji_scroll.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
    emoji_scroll.set_vexpand(true);
    emoji_scroll.set_hexpand(true);
    emoji_scroll.set_margin_top(4); // Reduced margin
    emoji_scroll.set_min_content_height(350); // Increased minimum height
    
    let emoji_flow = gtk4::FlowBox::new();
    emoji_flow.set_max_children_per_line(12);
    emoji_flow.set_min_children_per_line(8);
    emoji_flow.set_row_spacing(4); // Reduced spacing
    emoji_flow.set_column_spacing(4); // Reduced spacing
    emoji_flow.set_selection_mode(gtk4::SelectionMode::None);
    emoji_flow.set_homogeneous(true);
    emoji_flow.set_margin_top(4); // Reduced margin
    emoji_flow.set_margin_bottom(4);
    emoji_flow.set_margin_start(4);
    emoji_flow.set_margin_end(4);
    emoji_flow.set_valign(gtk4::Align::Start);
    emoji_flow.set_halign(gtk4::Align::Fill);
    
    emoji_scroll.set_child(Some(&emoji_flow));
    main_box.append(&emoji_scroll);
    
    // Emoji data structure
    let emoji_data = get_emoji_data();
    
    // Function to populate emoji grid
    let populate_emojis = {
        let emoji_flow = emoji_flow.clone();
        let editor = editor.clone();
        let dialog = dialog.clone();
        let emoji_data = emoji_data.clone();
        
        move |category: &str, search_term: &str| {
            // Clear existing children
            while let Some(child) = emoji_flow.first_child() {
                emoji_flow.remove(&child);
            }
            
            let search_lower = search_term.to_lowercase();
            
            for emoji in &emoji_data {
                // Filter by category
                if category != "all" && !emoji.categories.contains(&category.to_string()) {
                    continue;
                }
                
                // Filter by search term
                if !search_term.is_empty() {
                    let matches_search = emoji.name.to_lowercase().contains(&search_lower) ||
                                        emoji.keywords.iter().any(|k| k.to_lowercase().contains(&search_lower)) ||
                                        emoji.shortcode.to_lowercase().contains(&search_lower);
                    
                    if !matches_search {
                        continue;
                    }
                }
                
                // Create emoji button
                let emoji_button = gtk4::Button::with_label(&emoji.emoji);
                emoji_button.set_css_classes(&["emoji-button"]);
                emoji_button.set_tooltip_text(Some(&format!("{} ({})", emoji.name, emoji.shortcode)));
                
                let emoji_clone = emoji.clone();
                let editor_clone = editor.clone();
                let dialog_clone = dialog.clone();
                
                emoji_button.connect_clicked(move |_| {
                    editor_clone.insert_emoji_text(&emoji_clone.emoji);
                    dialog_clone.close();
                });
                
                emoji_flow.insert(&emoji_button, -1);
            }
        }
    };
    
    // Initial population
    populate_emojis("all", "");
    
    // Connect category buttons
    for (button, category) in &category_buttons {
        let populate_emojis = populate_emojis.clone();
        let category = category.clone();
        let search_entry = search_entry.clone();
        let other_buttons = category_buttons.clone();
        
        button.connect_toggled(move |btn| {
            if btn.is_active() {
                // Deactivate other buttons
                for (other_btn, other_cat) in &other_buttons {
                    if other_cat != &category {
                        other_btn.set_active(false);
                    }
                }
                
                let search_text = search_entry.text();
                populate_emojis(&category, &search_text);
            }
        });
    }
    
    // Connect search entry
    let search_populate = populate_emojis.clone();
    search_entry.connect_changed(move |entry| {
        let search_text = entry.text();
        let active_category = category_buttons.iter()
            .find(|(btn, _)| btn.is_active())
            .map(|(_, cat)| cat.as_str())
            .unwrap_or("all");
        
        search_populate(active_category, &search_text);
    });
    
    dialog.content_area().append(&main_box);
    
    // Add CSS for styling
    let css_provider = gtk4::CssProvider::new();
    css_provider.load_from_data(
        ".emoji-button { 
            font-size: 18px; 
            min-width: 36px; 
            min-height: 36px; 
            max-width: 36px;
            max-height: 36px;
            padding: 1px;
            margin: 1px;
            border-radius: 4px;
        }
        .category-button { 
            font-size: 14px; 
            min-width: 32px;
            min-height: 28px;
            padding: 2px 4px;
            margin: 1px;
            border-radius: 3px;
        }"
    );
    
    gtk4::style_context_add_provider_for_display(
        &gdk4::Display::default().unwrap(),
        &css_provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
    
    dialog.connect_response(|dialog, _| {
        dialog.close();
    });
    
    search_entry.grab_focus();
    dialog.present();
}

/// Get emoji data with categories, names, and shortcodes
fn get_emoji_data() -> Vec<EmojiData> {
    vec![
        // Faces & Emotions
        EmojiData::new("😀", "grinning face", ":grinning:", vec!["faces"], vec!["happy", "smile", "joy"]),
        EmojiData::new("😃", "grinning face with big eyes", ":smiley:", vec!["faces"], vec!["happy", "smile", "joy"]),
        EmojiData::new("😄", "grinning face with smiling eyes", ":smile:", vec!["faces"], vec!["happy", "joy"]),
        EmojiData::new("😁", "beaming face with smiling eyes", ":grin:", vec!["faces"], vec!["happy", "smile"]),
        EmojiData::new("😆", "grinning squinting face", ":laughing:", vec!["faces"], vec!["happy", "laugh"]),
        EmojiData::new("😅", "grinning face with sweat", ":sweat_smile:", vec!["faces"], vec!["happy", "laugh", "nervous"]),
        EmojiData::new("🤣", "rolling on the floor laughing", ":rofl:", vec!["faces"], vec!["laugh", "funny"]),
        EmojiData::new("😂", "face with tears of joy", ":joy:", vec!["faces"], vec!["laugh", "funny", "tears"]),
        EmojiData::new("😊", "smiling face with smiling eyes", ":blush:", vec!["faces"], vec!["happy", "shy"]),
        EmojiData::new("😇", "smiling face with halo", ":innocent:", vec!["faces"], vec!["angel", "good"]),
        EmojiData::new("😍", "smiling face with heart-eyes", ":heart_eyes:", vec!["faces"], vec!["love", "heart"]),
        EmojiData::new("🥰", "smiling face with hearts", ":smiling_face_with_hearts:", vec!["faces"], vec!["love", "heart"]),
        EmojiData::new("😘", "face blowing a kiss", ":kissing_heart:", vec!["faces"], vec!["kiss", "love"]),
        EmojiData::new("😗", "kissing face", ":kissing:", vec!["faces"], vec!["kiss"]),
        EmojiData::new("😙", "kissing face with smiling eyes", ":kissing_smiling_eyes:", vec!["faces"], vec!["kiss"]),
        EmojiData::new("😚", "kissing face with closed eyes", ":kissing_closed_eyes:", vec!["faces"], vec!["kiss"]),
        EmojiData::new("😋", "face savoring food", ":yum:", vec!["faces"], vec!["tasty", "food"]),
        EmojiData::new("😛", "face with tongue", ":stuck_out_tongue:", vec!["faces"], vec!["tongue", "silly"]),
        EmojiData::new("😜", "winking face with tongue", ":stuck_out_tongue_winking_eye:", vec!["faces"], vec!["tongue", "wink"]),
        EmojiData::new("🤪", "zany face", ":zany_face:", vec!["faces"], vec!["crazy", "silly"]),
        EmojiData::new("😝", "squinting face with tongue", ":stuck_out_tongue_closed_eyes:", vec!["faces"], vec!["tongue"]),
        EmojiData::new("🤑", "money-mouth face", ":money_mouth_face:", vec!["faces"], vec!["money", "rich"]),
        EmojiData::new("🤗", "hugging face", ":hugs:", vec!["faces"], vec!["hug", "love"]),
        EmojiData::new("🤭", "face with hand over mouth", ":hand_over_mouth:", vec!["faces"], vec!["secret", "quiet"]),
        EmojiData::new("🤫", "shushing face", ":shushing_face:", vec!["faces"], vec!["quiet", "secret"]),
        EmojiData::new("🤔", "thinking face", ":thinking:", vec!["faces"], vec!["think", "hmm"]),
        EmojiData::new("🤐", "zipper-mouth face", ":zipper_mouth_face:", vec!["faces"], vec!["quiet", "secret"]),
        EmojiData::new("🤨", "face with raised eyebrow", ":raised_eyebrow:", vec!["faces"], vec!["suspicious", "doubt"]),
        EmojiData::new("😐", "neutral face", ":neutral_face:", vec!["faces"], vec!["meh", "neutral"]),
        EmojiData::new("😑", "expressionless face", ":expressionless:", vec!["faces"], vec!["blank", "meh"]),
        EmojiData::new("😶", "face without mouth", ":no_mouth:", vec!["faces"], vec!["quiet", "silent"]),
        EmojiData::new("😏", "smirking face", ":smirk:", vec!["faces"], vec!["smug", "sly"]),
        EmojiData::new("😒", "unamused face", ":unamused:", vec!["faces"], vec!["annoyed", "meh"]),
        EmojiData::new("🙄", "face with rolling eyes", ":roll_eyes:", vec!["faces"], vec!["annoyed", "whatever"]),
        EmojiData::new("😬", "grimacing face", ":grimacing:", vec!["faces"], vec!["awkward", "nervous"]),
        EmojiData::new("🤥", "lying face", ":lying_face:", vec!["faces"], vec!["lie", "pinocchio"]),
        EmojiData::new("😔", "pensive face", ":pensive:", vec!["faces"], vec!["sad", "thoughtful"]),
        EmojiData::new("😪", "sleepy face", ":sleepy:", vec!["faces"], vec!["tired", "sleep"]),
        EmojiData::new("🤤", "drooling face", ":drooling_face:", vec!["faces"], vec!["drool", "want"]),
        EmojiData::new("😴", "sleeping face", ":sleeping:", vec!["faces"], vec!["sleep", "tired"]),
        EmojiData::new("😷", "face with medical mask", ":mask:", vec!["faces"], vec!["sick", "doctor"]),
        EmojiData::new("🤒", "face with thermometer", ":face_with_thermometer:", vec!["faces"], vec!["sick", "fever"]),
        EmojiData::new("🤕", "face with head-bandage", ":face_with_head_bandage:", vec!["faces"], vec!["hurt", "sick"]),
        EmojiData::new("🤢", "nauseated face", ":nauseated_face:", vec!["faces"], vec!["sick", "gross"]),
        EmojiData::new("🤮", "face vomiting", ":vomiting_face:", vec!["faces"], vec!["sick", "gross"]),
        EmojiData::new("🤧", "sneezing face", ":sneezing_face:", vec!["faces"], vec!["sick", "sneeze"]),
        EmojiData::new("🥵", "hot face", ":hot_face:", vec!["faces"], vec!["hot", "sweat"]),
        EmojiData::new("🥶", "cold face", ":cold_face:", vec!["faces"], vec!["cold", "freeze"]),
        EmojiData::new("🥴", "woozy face", ":woozy_face:", vec!["faces"], vec!["dizzy", "drunk"]),
        EmojiData::new("😵", "dizzy face", ":dizzy_face:", vec!["faces"], vec!["dizzy", "confused"]),
        EmojiData::new("🤯", "exploding head", ":exploding_head:", vec!["faces"], vec!["mind blown", "shock"]),
        EmojiData::new("😕", "confused face", ":confused:", vec!["faces"], vec!["confused", "puzzled"]),
        EmojiData::new("😟", "worried face", ":worried:", vec!["faces"], vec!["worried", "concerned"]),
        EmojiData::new("🙁", "slightly frowning face", ":slightly_frowning_face:", vec!["faces"], vec!["sad", "frown"]),
        EmojiData::new("☹️", "frowning face", ":frowning_face:", vec!["faces"], vec!["sad", "frown"]),
        EmojiData::new("😮", "face with open mouth", ":open_mouth:", vec!["faces"], vec!["surprised", "wow"]),
        EmojiData::new("😯", "hushed face", ":hushed:", vec!["faces"], vec!["surprised", "quiet"]),
        EmojiData::new("😲", "astonished face", ":astonished:", vec!["faces"], vec!["surprised", "shock"]),
        EmojiData::new("😳", "flushed face", ":flushed:", vec!["faces"], vec!["embarrassed", "shy"]),
        EmojiData::new("🥺", "pleading face", ":pleading_face:", vec!["faces"], vec!["puppy eyes", "please"]),
        EmojiData::new("😦", "frowning face with open mouth", ":frowning:", vec!["faces"], vec!["sad", "concerned"]),
        EmojiData::new("😧", "anguished face", ":anguished:", vec!["faces"], vec!["sad", "worried"]),
        EmojiData::new("😨", "fearful face", ":fearful:", vec!["faces"], vec!["scared", "fear"]),
        EmojiData::new("😰", "anxious face with sweat", ":cold_sweat:", vec!["faces"], vec!["nervous", "scared"]),
        EmojiData::new("😥", "sad but relieved face", ":disappointed_relieved:", vec!["faces"], vec!["sad", "relieved"]),
        EmojiData::new("😢", "crying face", ":cry:", vec!["faces"], vec!["sad", "tears"]),
        EmojiData::new("😭", "loudly crying face", ":sob:", vec!["faces"], vec!["sad", "cry", "tears"]),
        EmojiData::new("😱", "face screaming in fear", ":scream:", vec!["faces"], vec!["scared", "shock"]),
        EmojiData::new("😖", "confounded face", ":confounded:", vec!["faces"], vec!["frustrated", "angry"]),
        EmojiData::new("😣", "persevering face", ":persevere:", vec!["faces"], vec!["struggle", "effort"]),
        EmojiData::new("😞", "disappointed face", ":disappointed:", vec!["faces"], vec!["sad", "disappointed"]),
        EmojiData::new("😓", "downcast face with sweat", ":sweat:", vec!["faces"], vec!["sad", "tired"]),
        EmojiData::new("😩", "weary face", ":weary:", vec!["faces"], vec!["tired", "frustrated"]),
        EmojiData::new("😫", "tired face", ":tired_face:", vec!["faces"], vec!["tired", "exhausted"]),
        EmojiData::new("🥱", "yawning face", ":yawning_face:", vec!["faces"], vec!["tired", "sleepy"]),
        EmojiData::new("😤", "face with steam from nose", ":triumph:", vec!["faces"], vec!["angry", "mad"]),
        EmojiData::new("😡", "pouting face", ":rage:", vec!["faces"], vec!["angry", "mad"]),
        EmojiData::new("😠", "angry face", ":angry:", vec!["faces"], vec!["angry", "mad"]),
        EmojiData::new("🤬", "face with symbols on mouth", ":swearing:", vec!["faces"], vec!["angry", "curse"]),
        EmojiData::new("😈", "smiling face with horns", ":smiling_imp:", vec!["faces"], vec!["devil", "evil"]),
        EmojiData::new("👿", "angry face with horns", ":imp:", vec!["faces"], vec!["devil", "angry"]),
        EmojiData::new("💀", "skull", ":skull:", vec!["faces"], vec!["death", "dead"]),
        EmojiData::new("☠️", "skull and crossbones", ":skull_and_crossbones:", vec!["faces"], vec!["death", "danger"]),

        // Hearts & Love
        EmojiData::new("❤️", "red heart", ":heart:", vec!["hearts"], vec!["love", "red"]),
        EmojiData::new("🧡", "orange heart", ":orange_heart:", vec!["hearts"], vec!["love", "orange"]),
        EmojiData::new("💛", "yellow heart", ":yellow_heart:", vec!["hearts"], vec!["love", "yellow"]),
        EmojiData::new("💚", "green heart", ":green_heart:", vec!["hearts"], vec!["love", "green"]),
        EmojiData::new("💙", "blue heart", ":blue_heart:", vec!["hearts"], vec!["love", "blue"]),
        EmojiData::new("💜", "purple heart", ":purple_heart:", vec!["hearts"], vec!["love", "purple"]),
        EmojiData::new("🤎", "brown heart", ":brown_heart:", vec!["hearts"], vec!["love", "brown"]),
        EmojiData::new("🖤", "black heart", ":black_heart:", vec!["hearts"], vec!["love", "black"]),
        EmojiData::new("🤍", "white heart", ":white_heart:", vec!["hearts"], vec!["love", "white"]),
        EmojiData::new("💔", "broken heart", ":broken_heart:", vec!["hearts"], vec!["sad", "breakup"]),
        EmojiData::new("❣️", "heavy heart exclamation", ":heavy_heart_exclamation:", vec!["hearts"], vec!["love", "exclamation"]),
        EmojiData::new("💕", "two hearts", ":two_hearts:", vec!["hearts"], vec!["love", "couple"]),
        EmojiData::new("💞", "revolving hearts", ":revolving_hearts:", vec!["hearts"], vec!["love", "spinning"]),
        EmojiData::new("💓", "beating heart", ":heartbeat:", vec!["hearts"], vec!["love", "pulse"]),
        EmojiData::new("💗", "growing heart", ":heartpulse:", vec!["hearts"], vec!["love", "growing"]),
        EmojiData::new("💖", "sparkling heart", ":sparkling_heart:", vec!["hearts"], vec!["love", "sparkle"]),
        EmojiData::new("💘", "heart with arrow", ":cupid:", vec!["hearts"], vec!["love", "cupid"]),
        EmojiData::new("💝", "heart with ribbon", ":gift_heart:", vec!["hearts"], vec!["love", "gift"]),
        EmojiData::new("💟", "heart decoration", ":heart_decoration:", vec!["hearts"], vec!["love", "decoration"]),

        // People & Body
        EmojiData::new("👋", "waving hand", ":wave:", vec!["people"], vec!["hello", "goodbye", "hand"]),
        EmojiData::new("🤚", "raised back of hand", ":raised_back_of_hand:", vec!["people"], vec!["hand", "stop"]),
        EmojiData::new("🖐️", "hand with fingers splayed", ":raised_hand_with_fingers_splayed:", vec!["people"], vec!["hand", "five"]),
        EmojiData::new("✋", "raised hand", ":raised_hand:", vec!["people"], vec!["hand", "stop"]),
        EmojiData::new("🖖", "vulcan salute", ":vulcan_salute:", vec!["people"], vec!["hand", "spock"]),
        EmojiData::new("👌", "OK hand", ":ok_hand:", vec!["people"], vec!["hand", "ok", "good"]),
        EmojiData::new("🤌", "pinched fingers", ":pinched_fingers:", vec!["people"], vec!["hand", "italian"]),
        EmojiData::new("🤏", "pinching hand", ":pinching_hand:", vec!["people"], vec!["hand", "small"]),
        EmojiData::new("✌️", "victory hand", ":v:", vec!["people"], vec!["hand", "peace", "victory"]),
        EmojiData::new("🤞", "crossed fingers", ":crossed_fingers:", vec!["people"], vec!["hand", "luck", "hope"]),
        EmojiData::new("🤟", "love-you gesture", ":love_you_gesture:", vec!["people"], vec!["hand", "love"]),
        EmojiData::new("🤘", "sign of the horns", ":sign_of_the_horns:", vec!["people"], vec!["hand", "rock"]),
        EmojiData::new("🤙", "call me hand", ":call_me_hand:", vec!["people"], vec!["hand", "phone"]),
        EmojiData::new("👈", "backhand index pointing left", ":point_left:", vec!["people"], vec!["hand", "point", "left"]),
        EmojiData::new("👉", "backhand index pointing right", ":point_right:", vec!["people"], vec!["hand", "point", "right"]),
        EmojiData::new("👆", "backhand index pointing up", ":point_up_2:", vec!["people"], vec!["hand", "point", "up"]),
        EmojiData::new("🖕", "middle finger", ":middle_finger:", vec!["people"], vec!["hand", "rude"]),
        EmojiData::new("👇", "backhand index pointing down", ":point_down:", vec!["people"], vec!["hand", "point", "down"]),
        EmojiData::new("☝️", "index pointing up", ":point_up:", vec!["people"], vec!["hand", "point", "up"]),
        EmojiData::new("👍", "thumbs up", ":+1:", vec!["people"], vec!["hand", "good", "like"]),
        EmojiData::new("👎", "thumbs down", ":-1:", vec!["people"], vec!["hand", "bad", "dislike"]),
        EmojiData::new("✊", "raised fist", ":fist_raised:", vec!["people"], vec!["hand", "fist", "power"]),
        EmojiData::new("👊", "oncoming fist", ":fist_oncoming:", vec!["people"], vec!["hand", "fist", "punch"]),
        EmojiData::new("🤛", "left-facing fist", ":fist_left:", vec!["people"], vec!["hand", "fist", "left"]),
        EmojiData::new("🤜", "right-facing fist", ":fist_right:", vec!["people"], vec!["hand", "fist", "right"]),
        EmojiData::new("👏", "clapping hands", ":clap:", vec!["people"], vec!["hand", "applause", "good"]),
        EmojiData::new("🙌", "raising hands", ":raised_hands:", vec!["people"], vec!["hand", "celebration", "praise"]),
        EmojiData::new("👐", "open hands", ":open_hands:", vec!["people"], vec!["hand", "hug"]),
        EmojiData::new("🤲", "palms up together", ":palms_up_together:", vec!["people"], vec!["hand", "pray", "please"]),
        EmojiData::new("🤝", "handshake", ":handshake:", vec!["people"], vec!["hand", "deal", "agreement"]),
        EmojiData::new("🙏", "folded hands", ":pray:", vec!["people"], vec!["hand", "pray", "thanks"]),

        // Animals & Nature
        EmojiData::new("🐶", "dog face", ":dog:", vec!["animals"], vec!["dog", "puppy", "pet"]),
        EmojiData::new("🐱", "cat face", ":cat:", vec!["animals"], vec!["cat", "kitten", "pet"]),
        EmojiData::new("🐭", "mouse face", ":mouse:", vec!["animals"], vec!["mouse", "small"]),
        EmojiData::new("🐹", "hamster face", ":hamster:", vec!["animals"], vec!["hamster", "pet"]),
        EmojiData::new("🐰", "rabbit face", ":rabbit:", vec!["animals"], vec!["rabbit", "bunny"]),
        EmojiData::new("🦊", "fox face", ":fox_face:", vec!["animals"], vec!["fox", "clever"]),
        EmojiData::new("🐻", "bear face", ":bear:", vec!["animals"], vec!["bear", "strong"]),
        EmojiData::new("🐼", "panda face", ":panda_face:", vec!["animals"], vec!["panda", "cute"]),
        EmojiData::new("🐨", "koala", ":koala:", vec!["animals"], vec!["koala", "australia"]),
        EmojiData::new("🐯", "tiger face", ":tiger:", vec!["animals"], vec!["tiger", "strong"]),
        EmojiData::new("🦁", "lion face", ":lion:", vec!["animals"], vec!["lion", "king"]),
        EmojiData::new("🐮", "cow face", ":cow:", vec!["animals"], vec!["cow", "milk"]),
        EmojiData::new("🐷", "pig face", ":pig:", vec!["animals"], vec!["pig", "cute"]),
        EmojiData::new("🐸", "frog face", ":frog:", vec!["animals"], vec!["frog", "green"]),
        EmojiData::new("🐵", "monkey face", ":monkey_face:", vec!["animals"], vec!["monkey", "funny"]),
        EmojiData::new("🙈", "see-no-evil monkey", ":see_no_evil:", vec!["animals"], vec!["monkey", "hide"]),
        EmojiData::new("🙉", "hear-no-evil monkey", ":hear_no_evil:", vec!["animals"], vec!["monkey", "deaf"]),
        EmojiData::new("🙊", "speak-no-evil monkey", ":speak_no_evil:", vec!["animals"], vec!["monkey", "quiet"]),

        // Nature
        EmojiData::new("🌱", "seedling", ":seedling:", vec!["nature"], vec!["plant", "grow", "green"]),
        EmojiData::new("🌿", "herb", ":herb:", vec!["nature"], vec!["plant", "green", "leaf"]),
        EmojiData::new("🍀", "four leaf clover", ":four_leaf_clover:", vec!["nature"], vec!["luck", "green", "irish"]),
        EmojiData::new("🌳", "deciduous tree", ":deciduous_tree:", vec!["nature"], vec!["tree", "nature"]),
        EmojiData::new("🌲", "evergreen tree", ":evergreen_tree:", vec!["nature"], vec!["tree", "christmas"]),
        EmojiData::new("🌴", "palm tree", ":palm_tree:", vec!["nature"], vec!["tree", "tropical"]),
        EmojiData::new("🌵", "cactus", ":cactus:", vec!["nature"], vec!["desert", "spiky"]),
        EmojiData::new("🌸", "cherry blossom", ":cherry_blossom:", vec!["nature"], vec!["flower", "spring", "pink"]),
        EmojiData::new("🌺", "hibiscus", ":hibiscus:", vec!["nature"], vec!["flower", "tropical"]),
        EmojiData::new("🌻", "sunflower", ":sunflower:", vec!["nature"], vec!["flower", "yellow", "sun"]),
        EmojiData::new("🌹", "rose", ":rose:", vec!["nature"], vec!["flower", "love", "red"]),
        EmojiData::new("🌷", "tulip", ":tulip:", vec!["nature"], vec!["flower", "spring"]),
        EmojiData::new("🌼", "daisy", ":blossom:", vec!["nature"], vec!["flower", "white"]),

        // Food & Drink
        EmojiData::new("🍎", "red apple", ":apple:", vec!["food"], vec!["fruit", "healthy", "red"]),
        EmojiData::new("🍊", "tangerine", ":tangerine:", vec!["food"], vec!["fruit", "orange", "citrus"]),
        EmojiData::new("🍋", "lemon", ":lemon:", vec!["food"], vec!["fruit", "yellow", "sour"]),
        EmojiData::new("🍌", "banana", ":banana:", vec!["food"], vec!["fruit", "yellow", "monkey"]),
        EmojiData::new("🍉", "watermelon", ":watermelon:", vec!["food"], vec!["fruit", "summer", "sweet"]),
        EmojiData::new("🍇", "grapes", ":grapes:", vec!["food"], vec!["fruit", "purple", "wine"]),
        EmojiData::new("🍓", "strawberry", ":strawberry:", vec!["food"], vec!["fruit", "red", "sweet"]),
        EmojiData::new("🫐", "blueberries", ":blueberries:", vec!["food"], vec!["fruit", "blue", "healthy"]),
        EmojiData::new("🍈", "melon", ":melon:", vec!["food"], vec!["fruit", "green", "sweet"]),
        EmojiData::new("🍑", "cherries", ":cherries:", vec!["food"], vec!["fruit", "red", "sweet"]),
        EmojiData::new("🍒", "cherries", ":cherries:", vec!["food"], vec!["fruit", "red", "pair"]),
        EmojiData::new("🥭", "mango", ":mango:", vec!["food"], vec!["fruit", "tropical", "sweet"]),
        EmojiData::new("🍍", "pineapple", ":pineapple:", vec!["food"], vec!["fruit", "tropical", "spiky"]),
        EmojiData::new("🥥", "coconut", ":coconut:", vec!["food"], vec!["fruit", "tropical", "water"]),
        EmojiData::new("🥝", "kiwi fruit", ":kiwi_fruit:", vec!["food"], vec!["fruit", "green", "fuzzy"]),
        EmojiData::new("🍅", "tomato", ":tomato:", vec!["food"], vec!["vegetable", "red", "salad"]),
        EmojiData::new("🍆", "eggplant", ":eggplant:", vec!["food"], vec!["vegetable", "purple"]),
        EmojiData::new("🥑", "avocado", ":avocado:", vec!["food"], vec!["fruit", "green", "healthy"]),
        EmojiData::new("🥦", "broccoli", ":broccoli:", vec!["food"], vec!["vegetable", "green", "healthy"]),
        EmojiData::new("🥬", "leafy greens", ":leafy_greens:", vec!["food"], vec!["vegetable", "green", "salad"]),
        EmojiData::new("🥒", "cucumber", ":cucumber:", vec!["food"], vec!["vegetable", "green", "fresh"]),
        EmojiData::new("🌶️", "hot pepper", ":hot_pepper:", vec!["food"], vec!["spicy", "red", "hot"]),
        EmojiData::new("🫑", "bell pepper", ":bell_pepper:", vec!["food"], vec!["vegetable", "colorful"]),
        EmojiData::new("🌽", "ear of corn", ":corn:", vec!["food"], vec!["vegetable", "yellow"]),
        EmojiData::new("🥕", "carrot", ":carrot:", vec!["food"], vec!["vegetable", "orange", "healthy"]),
        EmojiData::new("🫒", "olive", ":olive:", vec!["food"], vec!["fruit", "green", "oil"]),
        EmojiData::new("🧄", "garlic", ":garlic:", vec!["food"], vec!["vegetable", "flavor", "white"]),
        EmojiData::new("🧅", "onion", ":onion:", vec!["food"], vec!["vegetable", "cry", "flavor"]),
        EmojiData::new("🥔", "potato", ":potato:", vec!["food"], vec!["vegetable", "brown"]),
        EmojiData::new("🍠", "roasted sweet potato", ":sweet_potato:", vec!["food"], vec!["vegetable", "orange"]),

        // Sports & Activities
        EmojiData::new("⚽", "soccer ball", ":soccer:", vec!["sports"], vec!["football", "sport", "ball"]),
        EmojiData::new("🏀", "basketball", ":basketball:", vec!["sports"], vec!["sport", "ball", "orange"]),
        EmojiData::new("🏈", "american football", ":football:", vec!["sports"], vec!["sport", "ball", "american"]),
        EmojiData::new("⚾", "baseball", ":baseball:", vec!["sports"], vec!["sport", "ball", "white"]),
        EmojiData::new("🥎", "softball", ":softball:", vec!["sports"], vec!["sport", "ball", "yellow"]),
        EmojiData::new("🎾", "tennis", ":tennis:", vec!["sports"], vec!["sport", "ball", "green"]),
        EmojiData::new("🏐", "volleyball", ":volleyball:", vec!["sports"], vec!["sport", "ball", "beach"]),
        EmojiData::new("🏉", "rugby football", ":rugby_football:", vec!["sports"], vec!["sport", "ball"]),
        EmojiData::new("🥏", "flying disc", ":flying_disc:", vec!["sports"], vec!["sport", "frisbee"]),
        EmojiData::new("🎱", "pool 8 ball", ":8ball:", vec!["sports"], vec!["sport", "billiards", "black"]),
        EmojiData::new("🪀", "yo-yo", ":yo_yo:", vec!["sports"], vec!["toy", "string"]),
        EmojiData::new("🏓", "ping pong", ":ping_pong:", vec!["sports"], vec!["sport", "table tennis"]),
        EmojiData::new("🏸", "badminton", ":badminton:", vec!["sports"], vec!["sport", "racket"]),
        EmojiData::new("🏒", "ice hockey", ":ice_hockey:", vec!["sports"], vec!["sport", "stick", "ice"]),
        EmojiData::new("🏑", "field hockey", ":field_hockey:", vec!["sports"], vec!["sport", "stick"]),
        EmojiData::new("🥍", "lacrosse", ":lacrosse:", vec!["sports"], vec!["sport", "stick"]),
        EmojiData::new("🏏", "cricket game", ":cricket_game:", vec!["sports"], vec!["sport", "bat"]),
        EmojiData::new("🪃", "boomerang", ":boomerang:", vec!["sports"], vec!["sport", "australia"]),

        // Travel & Places
        EmojiData::new("🚗", "automobile", ":car:", vec!["travel"], vec!["car", "vehicle", "red"]),
        EmojiData::new("🚕", "taxi", ":taxi:", vec!["travel"], vec!["car", "vehicle", "yellow"]),
        EmojiData::new("🚙", "sport utility vehicle", ":blue_car:", vec!["travel"], vec!["car", "vehicle", "blue"]),
        EmojiData::new("🚌", "bus", ":bus:", vec!["travel"], vec!["vehicle", "public"]),
        EmojiData::new("🚎", "trolleybus", ":trolleybus:", vec!["travel"], vec!["vehicle", "public"]),
        EmojiData::new("🏎️", "racing car", ":racing_car:", vec!["travel"], vec!["car", "fast", "sport"]),
        EmojiData::new("🚓", "police car", ":police_car:", vec!["travel"], vec!["car", "police", "law"]),
        EmojiData::new("🚑", "ambulance", ":ambulance:", vec!["travel"], vec!["vehicle", "medical", "emergency"]),
        EmojiData::new("🚒", "fire engine", ":fire_engine:", vec!["travel"], vec!["vehicle", "fire", "emergency"]),
        EmojiData::new("🚐", "minibus", ":minibus:", vec!["travel"], vec!["vehicle", "small"]),
        EmojiData::new("🛻", "pickup truck", ":pickup_truck:", vec!["travel"], vec!["vehicle", "truck"]),
        EmojiData::new("🚚", "delivery truck", ":truck:", vec!["travel"], vec!["vehicle", "delivery"]),
        EmojiData::new("🚛", "articulated lorry", ":articulated_lorry:", vec!["travel"], vec!["vehicle", "big"]),
        EmojiData::new("🚜", "tractor", ":tractor:", vec!["travel"], vec!["vehicle", "farm"]),
        EmojiData::new("🏍️", "motorcycle", ":motorcycle:", vec!["travel"], vec!["vehicle", "fast"]),
        EmojiData::new("🛵", "motor scooter", ":motor_scooter:", vec!["travel"], vec!["vehicle", "scooter"]),
        EmojiData::new("🛺", "auto rickshaw", ":auto_rickshaw:", vec!["travel"], vec!["vehicle", "asia"]),
        EmojiData::new("🚲", "bicycle", ":bike:", vec!["travel"], vec!["vehicle", "healthy"]),
        EmojiData::new("🛴", "kick scooter", ":kick_scooter:", vec!["travel"], vec!["vehicle", "scooter"]),
        EmojiData::new("🛹", "skateboard", ":skateboard:", vec!["travel"], vec!["vehicle", "sport"]),
        EmojiData::new("🛼", "roller skate", ":roller_skate:", vec!["travel"], vec!["vehicle", "sport"]),
        EmojiData::new("🚁", "helicopter", ":helicopter:", vec!["travel"], vec!["vehicle", "air"]),
        EmojiData::new("🛩️", "small airplane", ":small_airplane:", vec!["travel"], vec!["vehicle", "air"]),
        EmojiData::new("✈️", "airplane", ":airplane:", vec!["travel"], vec!["vehicle", "air", "fast"]),
        EmojiData::new("🛫", "airplane departure", ":airplane_departure:", vec!["travel"], vec!["air", "takeoff"]),
        EmojiData::new("🛬", "airplane arrival", ":airplane_arrival:", vec!["travel"], vec!["air", "landing"]),
        EmojiData::new("🪂", "parachute", ":parachute:", vec!["travel"], vec!["air", "sport"]),
        EmojiData::new("💺", "seat", ":seat:", vec!["travel"], vec!["airplane", "chair"]),
        EmojiData::new("🚀", "rocket", ":rocket:", vec!["travel"], vec!["space", "fast"]),
        EmojiData::new("🛸", "flying saucer", ":flying_saucer:", vec!["travel"], vec!["space", "ufo"]),

        // Objects & Symbols
        EmojiData::new("💡", "light bulb", ":bulb:", vec!["objects"], vec!["idea", "light", "bright"]),
        EmojiData::new("🔦", "flashlight", ":flashlight:", vec!["objects"], vec!["light", "torch"]),
        EmojiData::new("🕯️", "candle", ":candle:", vec!["objects"], vec!["light", "flame"]),
        EmojiData::new("🪔", "diya lamp", ":diya_lamp:", vec!["objects"], vec!["light", "oil"]),
        EmojiData::new("🧯", "fire extinguisher", ":fire_extinguisher:", vec!["objects"], vec!["fire", "safety"]),
        EmojiData::new("🛢️", "oil drum", ":oil_drum:", vec!["objects"], vec!["oil", "fuel"]),
        EmojiData::new("💰", "money bag", ":moneybag:", vec!["objects"], vec!["money", "rich", "bag"]),
        EmojiData::new("💴", "yen banknote", ":yen:", vec!["objects"], vec!["money", "japan"]),
        EmojiData::new("💵", "dollar banknote", ":dollar:", vec!["objects"], vec!["money", "usa"]),
        EmojiData::new("💶", "euro banknote", ":euro:", vec!["objects"], vec!["money", "europe"]),
        EmojiData::new("💷", "pound banknote", ":pound:", vec!["objects"], vec!["money", "uk"]),
        EmojiData::new("💳", "credit card", ":credit_card:", vec!["objects"], vec!["money", "card"]),
        EmojiData::new("💎", "gem stone", ":gem:", vec!["objects"], vec!["diamond", "precious"]),
        EmojiData::new("⚖️", "balance scale", ":balance_scale:", vec!["objects"], vec!["justice", "law"]),
        EmojiData::new("🦽", "manual wheelchair", ":manual_wheelchair:", vec!["objects"], vec!["accessibility", "medical"]),
        EmojiData::new("🦼", "motorized wheelchair", ":motorized_wheelchair:", vec!["objects"], vec!["accessibility", "medical"]),
        EmojiData::new("🩹", "adhesive bandage", ":adhesive_bandage:", vec!["objects"], vec!["medical", "hurt"]),
        EmojiData::new("🩺", "stethoscope", ":stethoscope:", vec!["objects"], vec!["medical", "doctor"]),
        EmojiData::new("💊", "pill", ":pill:", vec!["objects"], vec!["medical", "medicine"]),
        EmojiData::new("💉", "syringe", ":syringe:", vec!["objects"], vec!["medical", "shot"]),
        EmojiData::new("🌡️", "thermometer", ":thermometer:", vec!["objects"], vec!["medical", "temperature"]),
        EmojiData::new("🧿", "nazar amulet", ":nazar_amulet:", vec!["objects"], vec!["protection", "evil eye"]),
        EmojiData::new("🔮", "crystal ball", ":crystal_ball:", vec!["objects"], vec!["magic", "future"]),
        EmojiData::new("🪬", "hamsa", ":hamsa:", vec!["objects"], vec!["protection", "hand"]),
        EmojiData::new("🕳️", "hole", ":hole:", vec!["objects"], vec!["empty", "void"]),
        EmojiData::new("💣", "bomb", ":bomb:", vec!["objects"], vec!["explosive", "danger"]),
        EmojiData::new("🧨", "firecracker", ":firecracker:", vec!["objects"], vec!["explosive", "celebration"]),

        // Symbols & Numbers
        EmojiData::new("1️⃣", "keycap: 1", ":one:", vec!["symbols"], vec!["number", "1", "first"]),
        EmojiData::new("2️⃣", "keycap: 2", ":two:", vec!["symbols"], vec!["number", "2", "second"]),
        EmojiData::new("3️⃣", "keycap: 3", ":three:", vec!["symbols"], vec!["number", "3", "third"]),
        EmojiData::new("4️⃣", "keycap: 4", ":four:", vec!["symbols"], vec!["number", "4", "fourth"]),
        EmojiData::new("5️⃣", "keycap: 5", ":five:", vec!["symbols"], vec!["number", "5", "fifth"]),
        EmojiData::new("6️⃣", "keycap: 6", ":six:", vec!["symbols"], vec!["number", "6", "sixth"]),
        EmojiData::new("7️⃣", "keycap: 7", ":seven:", vec!["symbols"], vec!["number", "7", "seventh"]),
        EmojiData::new("8️⃣", "keycap: 8", ":eight:", vec!["symbols"], vec!["number", "8", "eighth"]),
        EmojiData::new("9️⃣", "keycap: 9", ":nine:", vec!["symbols"], vec!["number", "9", "ninth"]),
        EmojiData::new("🔟", "keycap: 10", ":ten:", vec!["symbols"], vec!["number", "10", "tenth"]),
        EmojiData::new("🔢", "input numbers", ":1234:", vec!["symbols"], vec!["numbers", "input"]),
        EmojiData::new("#️⃣", "keycap: #", ":hash:", vec!["symbols"], vec!["hashtag", "number"]),
        EmojiData::new("*️⃣", "keycap: *", ":asterisk:", vec!["symbols"], vec!["star", "multiply"]),
        EmojiData::new("⏏️", "eject button", ":eject_button:", vec!["symbols"], vec!["button", "media"]),
        EmojiData::new("▶️", "play button", ":arrow_forward:", vec!["symbols"], vec!["play", "media"]),
        EmojiData::new("⏸️", "pause button", ":pause_button:", vec!["symbols"], vec!["pause", "media"]),
        EmojiData::new("⏹️", "stop button", ":stop_button:", vec!["symbols"], vec!["stop", "media"]),
        EmojiData::new("⏺️", "record button", ":record_button:", vec!["symbols"], vec!["record", "media"]),
        EmojiData::new("⏭️", "next track button", ":track_next:", vec!["symbols"], vec!["next", "media"]),
        EmojiData::new("⏮️", "last track button", ":track_previous:", vec!["symbols"], vec!["previous", "media"]),
        EmojiData::new("⏯️", "play or pause button", ":play_or_pause_button:", vec!["symbols"], vec!["play", "pause", "media"]),
        EmojiData::new("🔀", "twisted rightwards arrows", ":twisted_rightwards_arrows:", vec!["symbols"], vec!["shuffle", "random"]),
        EmojiData::new("🔁", "repeat button", ":repeat:", vec!["symbols"], vec!["repeat", "loop"]),
        EmojiData::new("🔂", "repeat single button", ":repeat_one:", vec!["symbols"], vec!["repeat", "one"]),
        EmojiData::new("◀️", "reverse button", ":arrow_backward:", vec!["symbols"], vec!["reverse", "back"]),
        EmojiData::new("🔼", "upwards button", ":arrow_up_small:", vec!["symbols"], vec!["up", "triangle"]),
        EmojiData::new("🔽", "downwards button", ":arrow_down_small:", vec!["symbols"], vec!["down", "triangle"]),
        EmojiData::new("➡️", "right arrow", ":arrow_right:", vec!["symbols"], vec!["right", "direction"]),
        EmojiData::new("⬅️", "left arrow", ":arrow_left:", vec!["symbols"], vec!["left", "direction"]),
        EmojiData::new("⬆️", "up arrow", ":arrow_up:", vec!["symbols"], vec!["up", "direction"]),
        EmojiData::new("⬇️", "down arrow", ":arrow_down:", vec!["symbols"], vec!["down", "direction"]),
        EmojiData::new("↗️", "up-right arrow", ":arrow_upper_right:", vec!["symbols"], vec!["diagonal", "up-right"]),
        EmojiData::new("↘️", "down-right arrow", ":arrow_lower_right:", vec!["symbols"], vec!["diagonal", "down-right"]),
        EmojiData::new("↙️", "down-left arrow", ":arrow_lower_left:", vec!["symbols"], vec!["diagonal", "down-left"]),
        EmojiData::new("↖️", "up-left arrow", ":arrow_upper_left:", vec!["symbols"], vec!["diagonal", "up-left"]),
        EmojiData::new("↕️", "up-down arrow", ":arrow_up_down:", vec!["symbols"], vec!["vertical", "both"]),
        EmojiData::new("↔️", "left-right arrow", ":left_right_arrow:", vec!["symbols"], vec!["horizontal", "both"]),
        EmojiData::new("↩️", "right arrow curving left", ":leftwards_arrow_with_hook:", vec!["symbols"], vec!["return", "back"]),
        EmojiData::new("↪️", "left arrow curving right", ":arrow_right_hook:", vec!["symbols"], vec!["forward", "curve"]),
        EmojiData::new("⤴️", "right arrow curving up", ":arrow_heading_up:", vec!["symbols"], vec!["up", "curve"]),
        EmojiData::new("⤵️", "right arrow curving down", ":arrow_heading_down:", vec!["symbols"], vec!["down", "curve"]),
        EmojiData::new("🔃", "clockwise vertical arrows", ":arrows_clockwise:", vec!["symbols"], vec!["refresh", "reload"]),
        EmojiData::new("🔄", "counterclockwise arrows button", ":arrows_counterclockwise:", vec!["symbols"], vec!["refresh", "reverse"]),
        EmojiData::new("🔙", "BACK arrow", ":back:", vec!["symbols"], vec!["back", "return"]),
        EmojiData::new("🔚", "END arrow", ":end:", vec!["symbols"], vec!["end", "finish"]),
        EmojiData::new("🔛", "ON! arrow", ":on:", vec!["symbols"], vec!["on", "active"]),
        EmojiData::new("🔜", "SOON arrow", ":soon:", vec!["symbols"], vec!["soon", "coming"]),
        EmojiData::new("🔝", "TOP arrow", ":top:", vec!["symbols"], vec!["top", "best"]),
        EmojiData::new("✅", "check mark button", ":white_check_mark:", vec!["symbols"], vec!["check", "done", "yes"]),
        EmojiData::new("☑️", "check box with check", ":ballot_box_with_check:", vec!["symbols"], vec!["check", "done"]),
        EmojiData::new("✔️", "check mark", ":heavy_check_mark:", vec!["symbols"], vec!["check", "correct"]),
        EmojiData::new("❌", "cross mark", ":x:", vec!["symbols"], vec!["no", "wrong", "error"]),
        EmojiData::new("❎", "cross mark button", ":negative_squared_cross_mark:", vec!["symbols"], vec!["no", "wrong"]),
        EmojiData::new("➕", "plus sign", ":heavy_plus_sign:", vec!["symbols"], vec!["plus", "add", "more"]),
        EmojiData::new("➖", "minus sign", ":heavy_minus_sign:", vec!["symbols"], vec!["minus", "subtract", "less"]),
        EmojiData::new("➗", "division sign", ":heavy_division_sign:", vec!["symbols"], vec!["divide", "math"]),
        EmojiData::new("✖️", "multiplication sign", ":heavy_multiplication_x:", vec!["symbols"], vec!["multiply", "math"]),
        EmojiData::new("🟰", "heavy equals sign", ":heavy_equals_sign:", vec!["symbols"], vec!["equals", "math"]),
        EmojiData::new("♾️", "infinity", ":infinity:", vec!["symbols"], vec!["infinite", "forever"]),
        EmojiData::new("‼️", "double exclamation mark", ":bangbang:", vec!["symbols"], vec!["surprise", "emphasis"]),
        EmojiData::new("⁉️", "exclamation question mark", ":interrobang:", vec!["symbols"], vec!["surprise", "question"]),
        EmojiData::new("❓", "question mark", ":question:", vec!["symbols"], vec!["question", "confused"]),
        EmojiData::new("❔", "white question mark", ":grey_question:", vec!["symbols"], vec!["question", "help"]),
        EmojiData::new("❗", "exclamation mark", ":exclamation:", vec!["symbols"], vec!["surprise", "attention"]),
        EmojiData::new("❕", "white exclamation mark", ":grey_exclamation:", vec!["symbols"], vec!["surprise", "light"]),
        EmojiData::new("〰️", "wavy dash", ":wavy_dash:", vec!["symbols"], vec!["wave", "approximate"]),
        EmojiData::new("💱", "currency exchange", ":currency_exchange:", vec!["symbols"], vec!["money", "exchange"]),
        EmojiData::new("💲", "heavy dollar sign", ":heavy_dollar_sign:", vec!["symbols"], vec!["money", "dollar"]),
    ]
}

#[derive(Clone)]
struct EmojiData {
    emoji: String,
    name: String,
    shortcode: String,
    categories: Vec<String>,
    keywords: Vec<String>,
}

impl EmojiData {
    fn new(emoji: &str, name: &str, shortcode: &str, categories: Vec<&str>, keywords: Vec<&str>) -> Self {
        Self {
            emoji: emoji.to_string(),
            name: name.to_string(),
            shortcode: shortcode.to_string(),
            categories: categories.iter().map(|s| s.to_string()).collect(),
            keywords: keywords.iter().map(|s| s.to_string()).collect(),
        }
    }
}