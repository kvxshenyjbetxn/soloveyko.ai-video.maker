import requests
import base64
import time
import os
import sys

# ==========================================
# ⚙️ НАЛАШТУВАННЯ
# ==========================================

# 1. Твій ключ (ОБОВ'ЯЗКОВО ВСТАВ СЮДИ!)
API_KEY = "PWeLRZwXmKbgFPpywTi2icJBPFblUUkD" 

# 2. Файл з картинкою
IMAGE_PATH = "photo.jpg"

# 3. Промт
PROMPT = "Animate this scene, cinematic movement, 4k"

# Базовий URL з документації
BASE_URL = "https://googler.fast-gen.ai/api/v2"

# ==========================================

def encode_image(path):
    if not os.path.exists(path):
        print(f"❌ Файл {path} не знайдено!")
        input("Enter щоб вийти...")
        sys.exit()
    
    with open(path, "rb") as image_file:
        # v2 API вимагає Data URI формат (data:image/png;base64,...)
        encoded_string = base64.b64encode(image_file.read()).decode('utf-8')
        return f"data:image/jpeg;base64,{encoded_string}"

def main():
    print("🚀 Починаємо роботу по v2 API...")

    # 1. Готуємо заголовки (API Key вимагається в X-API-Key)
    headers = {
        "X-API-Key": API_KEY,
        "Content-Type": "application/json",
        "Accept": "application/json"
    }

    # 2. Кодуємо картинку
    print("📸 Кодую картинку...")
    image_data_uri = encode_image(IMAGE_PATH)

    # 3. Формуємо запит згідно з документацією (v2/videos)
    # Використовуємо операцію 'generate_video_from_image' (legacy) або 'generate_video_start_end'
    payload = {
        "provider": "google_fx",
        "operation": "generate_video_from_image", 
        "parameters": {
            "prompt": PROMPT,
            "input_image": image_data_uri,
            # "aspect_ratio": "VIDEO_ASPECT_RATIO_LANDSCAPE" # Можна додати якщо треба
        }
    }

    print(f"📡 Відправляю завдання на сервер...")
    
    try:
        response = requests.post(f"{BASE_URL}/videos", json=payload, headers=headers)
        
        if response.status_code != 200:
            print(f"❌ Помилка запуску: {response.status_code}")
            print(response.text)
            input("Enter...")
            return

        data = response.json()
        operation_id = data.get("operation_id")
        
        if not operation_id:
            print("❌ Не отримав operation_id! Щось пішло не так.")
            print(data)
            return

        print(f"✅ Завдання створено! ID: {operation_id}")
        print("⏳ Чекаємо виконання (Polling)...")

        # 4. ЦИКЛ ОЧІКУВАННЯ (Polling)
        while True:
            time.sleep(5) # Перевіряємо кожні 5 секунд
            
            status_url = f"{BASE_URL}/videos/status/{operation_id}"
            status_resp = requests.get(status_url, headers=headers)
            
            if status_resp.status_code != 200:
                print(f"⚠️ Помилка перевірки статусу: {status_resp.status_code}")
                continue
                
            status_data = status_resp.json()
            status = status_data.get("status")
            
            print(f"   ...Статус: {status}")

            if status == "success":
                # Ура, готово!
                result_base64 = status_data.get("result") # Або "output", перевіримо
                if not result_base64 and "output" in status_data: result_base64 = status_data["output"]
                
                print("💾 Відео готове! Завантажую...")
                
                # Чистимо заголовок data URI
                if "," in result_base64:
                    result_base64 = result_base64.split(",")[1]
                
                video_bytes = base64.b64decode(result_base64)
                
                filename = "FINAL_VIDEO_V2.mp4"
                with open(filename, "wb") as f:
                    f.write(video_bytes)
                
                print(f"\n✅✅✅ УСПІХ! Відео збережено як: {filename}")
                break
            
            elif status == "error":
                print("\n❌ Сервер повернув помилку при генерації!")
                print(status_data)
                break
            
            elif status in ["pending", "processing"]:
                # Просто чекаємо далі
                continue
            
            else:
                print(f"Невідомий статус: {status}")
                break

    except Exception as e:
        print(f"\n❌ Критична помилка скрипта: {e}")

    input("\n👉 Натисни Enter, щоб вийти...")

if __name__ == "__main__":
    main()