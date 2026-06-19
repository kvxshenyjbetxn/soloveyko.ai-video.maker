## graphify

This project has a knowledge graph at graphify-out/ with god nodes, community structure, and cross-file relationships.

Rules:
- For codebase questions, first run `graphify query "<question>"` when graphify-out/graph.json exists. Use `graphify path "<A>" "<B>"` for relationships and `graphify explain "<concept>"` for focused concepts. These return a scoped subgraph, usually much smaller than GRAPH_REPORT.md or raw grep output.
- If graphify-out/wiki/index.md exists, use it for broad navigation instead of raw source browsing.
- Read graphify-out/GRAPH_REPORT.md only for broad architecture review or when query/path/explain do not surface enough context.
- After modifying code, run `graphify update .` to keep the graph current (AST-only, no API cost).


- не роби коміти до поки я не скажу.
Before editing any file, read it first. Before modifying a function, grep for all callers. Research before you edit.
- відповідаеш завжди українською мовою.
- стек програми rust + egui
- пишеш простий та лаконічний код, який легко читати та розуміти.
- використовуєш функції та структури для організації коду.
- коментуєш код, щоб пояснити його логіку та призначення.
- підтримуеш чітку структуру проекту, розділяючи код на модулі та файли.
- перевіряеш код на наявність помилок та оптимізуєш його для кращої продуктивності.
- використовуєш сучасні можливості Rust для забезпечення безпеки та ефективності.
- дотримуєшся принципів чистого коду, уникаючи надмірної складності та дублювання.
- не засмічуеш код, якщо функція вже не використовуеться то видаляеш її, а не залишаєш закоментованою.
- використовуєш бібліотеки та інструменти, які допомагають спростити розробку та покращити якість коду.
- программа мае бути адаптована під Windows та MacOS, завжди перевіряй цю крос платформеність.
- при додаванні нових текстів або елементів інтерфейсу, то не забувай перекладати їх на всі мови інтерфейсу - /soloveyko.ai-video.maker/src/localization
- при видалення якихось елементів не забудь видалити ключа перекладу з коду.
- при додаванні нових елементів в панель пайплайну не забувай додавати ці налаштування в збереження в шаблон, ось інструкція (Ось чітка схема, куди саме їх дописувати:                                   
                                                                              
  ### 1. Додавання полів у структуру шаблону ( src/gui/settings/storage.rs )  
                                                                              
  Спочатку потрібно додати нові поля у саму структуру  PipelineTemplate , яка 
  записується в JSON.                                                         
                                                                              
    // src/gui/settings/storage.rs                                            
                                                                              
    #[derive(serde::Serialize, serde::Deserialize, Clone, Default)]           
    pub struct PipelineTemplate {                                             
        pub openrouter_key: String,                                           
                                                                              
        // СЮДИ додаєш будь-які нові налаштування пайплайну:                  
        // pub voice_id: String,       // Наприклад, обраний голос            
        // pub video_resolution: String, // Наприклад, роздільна здатність    
    }                                                                         
                                                                              
  ### 2. Оновлення функції збереження ( src/gui/settings/storage.rs )         
                                                                              
  Далі оновлюємо функцію  save_template , щоб вона приймала ці нові значення  
  та записувала їх у JSON:                                                    
                                                                              
    // src/gui/settings/storage.rs                                            
  
    pub fn save_template(
        name: &str, 
        openrouter_key: &str,
        // voice_id: &str, // додаємо новий аргумент
    ) -> Result<(), std::io::Error> {
        ...
        let template = PipelineTemplate {
            openrouter_key: openrouter_key.to_string(),
            // voice_id: voice_id.to_string(), // записуємо у структуру       
        };
        ...
    }
  
  ### 3. Передача значень при збереженні ( src/gui/pipeline/mod.rs )          
  
  У верху панелі пайплайну, де обробляється клік по кнопці Зберегти, передаємо
  поточні значення інтерфейсу у функцію збереження:
  
    // src/gui/pipeline/mod.rs
  
    if save_btn.clicked() {
        let name = template_name_input.trim();
        ...
        // Передаємо поточні змінні стану програми (наприклад, openrouter_key,
  voice_id тощо)
        match crate::gui::settings::storage::save_template(name,
  openrouter_key /*, voice_id */) {
            Ok(_) => { ... }
        }
    }
  
  ### 4. Застосування налаштувань при завантаженні ( 
  src/gui/pipeline/templates.rs )
  
  Коли користувач клікає по збереженому шаблону у списку, ми дістаємо значення
  з файлу та оновлюємо поточний стан програми:
  
    // src/gui/pipeline/templates.rs
  
    if btn.clicked() {
        if let Some(template) =
  crate::gui::settings::storage::load_template(&template_name) {
            // Оновлюємо поточні змінні програми новими значеннями з шаблону  
            *openrouter_key = template.openrouter_key;
            // *voice_id = template.voice_id; // застосовуємо нове значення   
        }
    }
  
  Таким чином, для будь-якого нового налаштування пайплайну достатньо         
  розширити
  структуру  PipelineTemplate , прокинути нове поле через функцію збереження  
  та застосувати його при завантаженні.)

- всі сервіси апі та виклики апі знаходяться в папці src/api, якщо потрібно додати новий сервіс, то створюєш новий файл для цього сервісу, наприклад src/api/new_service.rs