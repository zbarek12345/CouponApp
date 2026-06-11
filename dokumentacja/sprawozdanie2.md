# Raport projektu: Aplikacja multimedialna "Coupon & Receipt Manager"

## 1. Wprowadzenie

Projekt dotyczy aplikacji mobilnej i desktopowej zbudowanej przy użyciu **Tauri** oraz **Vue.js**, której głównym celem jest ułatwienie użytkownikom zbierania, organizowania i szybkiego odnajdywania kuponów promocyjnych, a także zarządzania paragonami i przypisywania ich do konkretnych sklepów. 

Zgodnie z wymaganiami projektu multimedialnego, aplikacja prezentuje zbiór elementów (traktowanych jako "produkty"), które są zorganizowane w logiczne kategorie. Interfejs wykorzystuje grafikę, tekst, płynne animacje przejść oraz zapewnia przestrzeń na integrację materiałów wideo/audio (np. animowane instrukcje obsługi lub klipy wideo dołączane do konkretnych ofert).

---

## 2. Podział na kategorie i produkty (Wymóg 12 produktów / 3 kategorie)

Aplikacja opiera się na strukturze bazy danych (SQLite), która kategoryzuje treści. Aby spełnić wymogi specyfikacji, system prezentuje łącznie ponad 12 "produktów" podzielonych na 3 główne kategorie:

1. **Kategoria 1: Sklepy (Shops)**
   * *Sklepik osiedlowy* (z własnym logo i zbiorem kuponów)
   * *Tesco* (hipermarket)
   * *Żabka* (sklep convenience)
   * *Biedronka* (dyskont)
2. **Kategoria 2: Kupony Promocyjne (Coupons)**
   * Kupon -20% na napoje gazowane
   * Kupon 2+1 gratis na przekąski
   * Rabat jednorazowy 50 PLN na elektronikę
   * Zniżka lojalnościowa dla stałych klientów
3. **Kategoria 3: Paragony i Dowody Zakupu (Receipts)**
   * Paragon z zakupów spożywczych (zawiera zidentyfikowane pozycje z OCR)
   * Paragon za sprzęt AGD (z dłuższą gwarancją)
   * Faktura za usługi
   * Szybki bilet z kasy fiskalnej

---

## 3. Technologie i Multimedia

| Technologia                          | Zastosowanie                                                        |
| ------------------------------------ | ------------------------------------------------------------------- |
| **Tauri & Rust** | Warstwa backendowa, obsługa bazy SQLite, skanowanie kodów QR, generowanie obrazów Base64. |
| **Vue.js 3** | Reaktywny interfejs użytkownika (Composition API). |
| **Grafika i Wideo** | Natywny interfejs DOM renderujący pliki PNG/JPEG, z miejscem na tagi `<video>` (H.264) i karuzele multimedialne. |
| **Animacje (CSS/JS)** | Własnoręcznie zaimplementowane płynne przejścia (`transition: opacity, transform`) oraz gesty typu Swipe. |

---

## 4. Wymagania projektowe (Funkcjonalne i Niefunkcjonalne)

### Wymagania funkcjonalne
* **Zarządzanie bazą:** Możliwość tworzenia, odczytu i usuwania sklepów, kuponów oraz paragonów.
* **Skanowanie i generowanie kodów:** Wczytywanie kodów z obrazów oraz renderowanie ich zwrotne do interfejsu (moduł *rxing* w Rust).
* **Personalizacja (Wymóg 24a):** Użytkownik posiada panel "Settings", w którym może w czasie rzeczywistym zmieniać motyw (Jasny/Ciemny), akcenty kolorystyczne (Ocean, Ember, Forest, Graphite) oraz krój i rozmiar fontu.
* **Integracja multimediów:** Wyświetlanie logotypów zapisanych lokalnie w katalogach aplikacji (AppData) oraz wstrzykiwanie ich w formacie Base64 do widoków Vue.

### Wymagania niefunkcjonalne
* **Wydajność:** Wykorzystanie lekkiego backendu w języku Rust zapewnia minimalne opóźnienia i natychmiastowe ładowanie bazy danych.
* **Responsywność (Wymóg 26c):** Aplikacja automatycznie dostosowuje interfejs do rozmiaru ekranu (wykorzystanie CSS Grid). Na urządzeniach mobilnych pasek zakładek znajduje się na dole (uwzględniając `safe-area-inset-bottom`), natomiast na szerokich ekranach (>768px) zamienia się w boczny panel nawigacyjny.
* **Bezpieczeństwo:** Dane (paragony, kupony) trzymane są w lokalnej bazie SQLite (`coupon_app.db`), bez wysyłania ich na zewnętrzne serwery.

---

## 5. Typografia, Formaty Plików i Kompresja

Projekt został ujednolicony pod kątem standardów plików oraz krojów pisma:

* **Typografia (CSS Variables):** Zaimplementowano cztery dynamiczne rodziny krojów pisma do wyboru przez użytkownika:
  * *System* (`Inter, ui-sans-serif, system-ui...`)
  * *Rounded* (`Trebuchet MS, Segoe UI...`)
  * *Serif* (`Georgia, Times New Roman, serif`)
  * *Mono* (`ui-monospace, SFMono-Regular, monospace`)
  * Rozmiar bazowy (font-size) jest w pełni skalowalny z poziomu ustawień (zakres 14px – 22px).
* **Formaty Graficzne i Kodeki:**
  * **Obrazy (Logotypy, Skanowane paragony):** Użyto formatu **PNG** ze względu na bezstratną kompresję, co jest kluczowe dla skanowania kodów QR i OCR. Silnik Rust dekoduje grafiki poprzez standardowe biblioteki (np. z formatu Base64 na strumień bajtów).
  * **Ikony interfejsu:** Wykorzystano wektorowy format **SVG** wbudowany bezpośrednio w komponenty Vue, gwarantujący idealną ostrość niezależnie od gęstości pikseli (PPI).
  * **Wideo/Audio:** W dołączanych multimedialnych elementach demonstracyjnych zakłada się wykorzystanie formatu **MP4 z kodekiem wideo H.264** oraz kodekiem audio **AAC**. Zapewnia to maksymalną kompatybilność w widokach WebView wykorzystywanych przez Tauri.

---

## 6. Zasady UI/UX i Animacje

Aplikacja ściśle trzyma się dobrych praktyk UI/UX:
* **Czytelna hierarchia:** Użyto modelu kart (Cards), co pozwala wyodrębnić każdy kupon i paragon. 
* **Karuzela/Przesuwanie (Wymóg 29f):** Zaimplementowano skrypt wykrywający gesty przesunięcia ekranu palcem (Swipe: `touchstart`, `touchend`). Użytkownik może przesuwać palcem w lewo/prawo, aby dynamicznie przechodzić między sklepami, kuponami a paragonami.
* **Autorska Animacja (Wymóg 25b):**
  Aplikacja posiada zaprogramowaną, autorską animację w CSS. Przycisk ustawień (zębatka) posiada złożoną animację zmieniającą dwa parametry – **rozmiar oraz kolor/rotację**:
  `transform: rotate(20deg) scale(0.95); background: rgba(255, 255, 255, 0.3); border-color: rgba(255, 255, 255, 0.5);` 
  Zastosowano również płynne wejścia/wyjścia zakładek za pomocą komponentu `<transition name="tab-slide">`, który modyfikuje przezroczystość (opacity) oraz przesunięcie (transform).

---

## 7. Formularz projektu (Wizualizacje)

### 7.1. Logo oraz Ulotka
| Logo Projektu | Ulotka Informacyjna |
| :---: | :---: |
| ![Logo projektu](logo_projektu.jpg) | ![Ulotka projektu](ulotka_projektu.jpg) |

### 7.2. Szablon aplikacji (Wireframe)

| Widok sklepów | Widok kuponów |
| :---: | :---: |
| ![Widok sklepów](Wireframe_Shops.png) | ![Widok kuponów](Wireframe_Coupons.png) |
| Struktura kafelków sklepów, główny obszar na treść, dolny pasek nawigacji. | Karuzela/karty kuponów z wyraźnym wyróżnieniem stanu pustego (Empty State). |

| Widok paragonów | Widok ustawień |
| :---: | :---: |
| ![Widok paragonów](Wireframe_receipts.png) | ![Widok ustawień](Wireframe_settings.png) |
| System paginacji oraz podglądu skanowania. | Sekcja personalizacji motywów i typografii. |

### 7.3. Makieta (Mockup) działającego interfejsu

| Widok sklepów | Widok kuponów |
| :---: | :---: |
| ![Widok sklepów](shop-view.png) | ![Widok kuponów](Coupons-view.png) |

| Widok paragonów | Widok ustawień |
| :---: | :---: |
| ![Widok paragonów](receipt-view.png) | ![Widok ustawień](settings-view.png) |

---

## 8. Testy z instrumentacją (QA)

Aplikacja przeszła proces testowania z instrumentacją, obejmujący zachowanie systemu pod różnymi obciążeniami i w różnych środowiskach:
* **Responsywność i ekran końcowy:** Testowano skalowanie okna aplikacji oraz symulowano rozdzielczości urządzeń mobilnych (np. proporcje 16:9, 18:9, 19.5:9). CSS Grid poprawnie refaktorował układ bez "łamania" interfejsu. Weryfikowano działanie na wysokich gęstościach pikseli (HiDPI / Retina), sprawdzając ostrość czcionek systemowych oraz wektorów SVG.
* **Orientacja:** Położenie horyzontalne automatycznie wykorzystuje dodatkową przestrzeń na rozszerzenie list, a układ przełącza się z paska dolnego na kolumnę boczną.
* **Automatyzacja:** Backend napisany w Rust został poddany unit-testom obejmującym dekodowanie plików (np. zamiana PNG Base64 na surowe dane do bazy), logikę bazy danych (SQLite CRUD) oraz parsowanie bloków OCR (Bounding Boxes z ujemnymi współrzędnymi).

---

## 9. Analiza praw autorskich

W ramach projektu przeprowadzono audyt praw autorskich:
* **Kod źródłowy:** Biblioteki takie jak Vue.js, Tauri, `sqlx` (Rust), oraz biblioteki skanujące np. `rxing` udostępniane są na liberalnych licencjach (np. MIT, Apache 2.0). Pozwala to na dowolne modyfikowanie i użycie w projektach uczelnianych/komercyjnych.
* **Typografia i Fonty:** Użyto krojów wbudowanych w system operacyjny (`system-ui`, `Segoe UI`, `SFMono`). Nie ma potrzeby załączania zewnętrznych plików .ttf/.woff obciążonych restrykcyjnymi licencjami.
* **Grafika (Ikony/Logo):** Wykorzystano generyczne ikony SVG tworzone własnoręcznie lub pochodzące ze zbiorów na licencji open-source. Autorskie logo oraz makiety nie naruszają praw autorskich podmiotów trzecich.
* **Multimedia (Audio/Wideo):** Opcjonalne grafiki produktów i darmowe efekty dźwiękowe (o ile włączone w ostatecznym buildzie) pochodzą wyłącznie ze źródeł Creative Commons Zero (CC0) lub licencji Pixabay/Pexels. 
Wykazano, że projekt zachowuje pełną czystość licencyjną.

---

## 10. Podsumowanie

Aplikacja multimedialna "Coupon & Receipt Manager" stanowi kompletną realizację założeń projektu z obszaru systemów multimedialnych. Łączy w sobie responsywny i nowoczesny interfejs (Vue.js) z wysoce wydajnym silnikiem natywnym (Tauri/Rust). Integracja grafiki, personalizacja typografii i kolorystyki, zaprogramowane od zera animacje, zaimplementowana obsługa gestów oraz udokumentowana architektura bazodanowa dla zebranych produktów i kategorii stanowią w pełni funkcjonalny prototyp spełniający wszystkie wymagania funkcjonalne, jak i niefunkcjonalne.