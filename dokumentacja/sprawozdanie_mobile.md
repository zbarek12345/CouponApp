# Raport projektu: aplikacja mobilna do zbierania kuponów i skanowania kodów QR



## 1. Wprowadzenie



Projekt dotyczy aplikacji mobilnej zbudowanej przy użyciu **Tauri** oraz **Vue.js**, której głównym celem jest ułatwienie użytkownikom zbierania, organizowania i szybkiego odnajdywania kuponów promocyjnych. Aplikacja pozwala również na skanowanie kodów QR, dodawanie sklepów oraz zarządzanie kuponami w sposób wygodny i uporządkowany.



Rozwiązanie zostało zaprojektowane z myślą o użytkownikach, którzy często korzystają z promocji, kart lojalnościowych lub kuponów rabatowych i chcą mieć do nich szybki dostęp w jednym miejscu.



---



## 2. Technologie wykorzystane w projekcie



Aplikacja została wykonana w oparciu o następujące technologie:



| Technologia                          | Zastosowanie                                                        |
| ------------------------------------ | ------------------------------------------------------------------- |
| **Tauri**                            | Warstwa aplikacyjna, integracja z systemem oraz budowanie aplikacji |
| **Rust**                             | Logika backendowa aplikacji oraz obsługa funkcji natywnych          |
| **Vue.js**                           | Interfejs użytkownika                                               |
| **JavaScript / TypeScript**          | Logika po stronie frontendu                                         |
| **HTML / CSS**                       | Struktura i stylizacja widoków                                      |
| **QR Scanner**                       | Obsługa skanowania kodów QR                                         |
| **Moduł OCR / skanowania paragonów** | Rozpoznawanie danych z paragonów, dostępne wyłącznie na komputerze  |



Całość aplikacji opiera się na ekosystemie **Tauri**, dlatego część natywna projektu została przygotowana w języku **Rust**. Pozwala to uzyskać dobrą wydajność, niski narzut zasobów oraz większą kontrolę nad działaniem aplikacji.



---



## 3. Cel aplikacji



Głównym celem aplikacji jest maksymalne skrócenie czasu potrzebnego na odnalezienie odpowiedniego kuponu podczas zakupów. Użytkownik może przechowywać kupony w jednym miejscu, przypisywać je do konkretnych sklepów oraz szybko wyszukiwać interesujące promocje.



Aplikacja rozwiązuje problem rozproszenia kuponów pomiędzy różnymi aplikacjami, stronami internetowymi, wiadomościami e-mail lub zdjęciami zapisanymi w galerii telefonu.



---



## 4. Główne funkcje aplikacji



### 4.1. Zbieranie kuponów



Aplikacja pozwala użytkownikowi dodawać i przechowywać kupony promocyjne. Każdy kupon może zawierać podstawowe informacje, takie jak:



* nazwa kuponu,

* sklep, którego dotyczy kupon,

* kod kuponu,

* data ważności,

* opis promocji,

* zdjęcie lub grafika kuponu,

* kod QR lub kod kreskowy.



Dzięki temu użytkownik może szybko sprawdzić, jakie kupony posiada i które z nich są nadal ważne.



---



### 4.2. Skanowanie kodów QR



Jedną z kluczowych funkcji aplikacji jest możliwość skanowania kodów QR. Funkcja ta może być wykorzystywana między innymi do:



* dodawania nowych kuponów,

* odczytywania kodów rabatowych,

* szybkiego przechodzenia do stron promocyjnych,

* identyfikowania kuponów przypisanych do konkretnego sklepu.



Skanowanie kodów QR znacząco przyspiesza korzystanie z aplikacji, ponieważ użytkownik nie musi ręcznie przepisywać kodów ani danych promocyjnych.



---



### 4.3. Dodawanie sklepów



Aplikacja umożliwia dodawanie sklepów, do których można przypisywać kupony. Jest to jedna z najważniejszych funkcji organizacyjnych całego projektu.



Użytkownik może utworzyć własną listę sklepów, a następnie przypisywać do nich konkretne kupony. Dzięki temu odnalezienie odpowiedniego kuponu podczas zakupów zajmuje bardzo mało czasu.



Przykładowe dane sklepu:



* nazwa sklepu,

* logo sklepu,

* kategoria sklepu,

* lista przypisanych kuponów,

* opcjonalny opis lub notatka.



---



### 4.4. Szybkie wyszukiwanie kuponów



Dzięki powiązaniu kuponów ze sklepami aplikacja pozwala bardzo szybko odnaleźć potrzebną promocję. Zamiast przeglądać wszystkie dostępne kupony, użytkownik może wybrać konkretny sklep i od razu zobaczyć tylko te kupony, które są z nim związane.



Funkcja ta jest szczególnie przydatna podczas zakupów, gdy liczy się czas i wygoda.



---



### 4.5. Moduł skanowania paragonów



Projekt zawiera również moduł skanowania paragonów. Funkcja ta pozwala analizować paragony i potencjalnie wykorzystywać dane zakupowe do dalszej organizacji kuponów lub historii zakupów.



Niestety, moduł ten działa **wyłącznie na komputerze**. Oznacza to, że nie jest dostępny w pełnej wersji mobilnej aplikacji. Wynika to z ograniczeń technicznych związanych z obsługą skanowania, przetwarzania obrazu lub użytej biblioteki OCR.



Mimo tego moduł stanowi ważny element projektu, ponieważ pokazuje możliwość dalszego rozwoju aplikacji w kierunku automatycznego analizowania zakupów i dopasowywania kuponów do rzeczywistych potrzeb użytkownika.



---



## 5. Architektura aplikacji



Aplikacja została oparta na architekturze charakterystycznej dla projektów Tauri.



```text
Frontend (Vue.js)
	|
	| komunikacja z API Tauri
	v
Backend (Rust)
	|
	| dostęp do funkcji systemowych
	v
System / pliki / kamera / moduły natywne
```



### 5.1. Frontend



Frontend aplikacji został przygotowany w **Vue.js**. Odpowiada on za:



* wyświetlanie listy kuponów,

* obsługę formularzy,

* widoki sklepów,

* ekran skanowania kodów QR,

* interakcję użytkownika z aplikacją,

* prezentację danych w czytelnej formie.



Vue.js pozwala na budowanie dynamicznych komponentów, dzięki czemu interfejs aplikacji może być prosty, przejrzysty i wygodny w użyciu.



### 5.2. Backend



Backend aplikacji został wykonany w **Rust**, ponieważ Tauri wykorzystuje Rust jako główny język części natywnej. Warstwa backendowa odpowiada za:



* obsługę danych,

* komunikację z systemem,

* integrację z modułami natywnymi,

* obsługę skanowania,

* potencjalne zapisywanie danych lokalnie,

* przetwarzanie informacji z kuponów i paragonów.



Rust zapewnia wysoką wydajność oraz bezpieczeństwo pamięci, co jest istotne w aplikacjach działających lokalnie na urządzeniu użytkownika.



---



## 6. Formularz projektu



Poniżej znajduje się miejsce na materiały wizualne, mockupy oraz zdjęcia właściwego projektu.
### 6.1. Ulotka informacyjna projektu
Poniżej przedstawiono ulotkę informacyjną projektu.

![Ulotka projektu](ulotka_projektu.png)




### 6.2. Szablon aplikacji (wireframe)


| Widok sklepów | Widok kuponów |
| --- | --- |
| ![Widok sklepów](Wireframe_Shops.png) | ![Widok kuponów](Wireframe_Coupons.png) |
| Lista sklepów z logo, liczbą kuponów i szybkim przyciskiem dodania nowego sklepu. | Karty kuponów z nazwą, sklepem, datą ważności i kodem QR. |

| Widok paragonów | Widok ustawień |
| --- | --- |
| ![Widok paragonów](Wireframe_receipts.png) | ![Widok ustawień](Wireframe_settings.png) |
| Lista paragonów z kwotą i datą zakupu oraz szybkim podglądem szczegółów. | Ustawienia aplikacji: preferencje kolorystyki, czcionek oraz wyglądu. |



### 6.3. Makieta (mockup) aplikacji

| Widok sklepów | Widok kuponów |
| --- | --- |
| ![Widok sklepów](shop-view.png) | ![Widok kuponów](Coupons-view.png) |
| Lista sklepów z logo, liczbą kuponów i szybkim przyciskiem dodania nowego sklepu. | Karty kuponów z nazwą, sklepem, datą ważności i kodem QR. |

| Widok paragonów | Widok ustawień |
| --- | --- |
| ![Widok paragonów](receipt-view.png) | ![Widok ustawień](settings-view.png) |
| Lista paragonów z kwotą i datą zakupu oraz szybkim podglądem szczegółów. | Ustawienia aplikacji: preferencje powiadomień, skanera oraz wyglądu. |

---



### 6.4. Zdjęcia właściwego projektu



> **Miejsce na zdjęcia gotowego projektu**



![Ekran główny aplikacji](ścieżka/do/zdjecia-1.png)



![Widok listy kuponów](ścieżka/do/zdjecia-2.png)



![Widok skanowania kodu QR](ścieżka/do/zdjecia-3.png)



![Widok dodawania sklepu](ścieżka/do/zdjecia-4.png)


Opis zdjęć:




---



## 7. Przykładowe widoki aplikacji



### 7.1. Ekran główny



Ekran główny może zawierać listę najważniejszych funkcji aplikacji, takich jak:



* lista ostatnio dodanych kuponów,

* szybki dostęp do skanera QR,

* przycisk dodawania nowego kuponu,

* sekcja popularnych lub ostatnio używanych sklepów.



---


### 7.2. Widok kuponów



Widok kuponów umożliwia przeglądanie zapisanych promocji. Każdy kupon powinien być przedstawiony w formie czytelnej karty zawierającej nazwę, sklep, datę ważności oraz kod.



Przykładowa karta kuponu:



```text

Nazwa: -20% na zakupy

Sklep: Przykładowy sklep

Kod: PROMO20

Ważny do: 31.12.2026

```



---



### 7.3. Widok sklepów



Widok sklepów pozwala użytkownikowi zarządzać miejscami, do których przypisane są kupony. Po wybraniu sklepu użytkownik widzi tylko kupony związane z tym konkretnym miejscem.



Dzięki temu aplikacja ogranicza czas potrzebny na znalezienie odpowiedniego kuponu.



---



### 7.4. Widok skanera QR



Widok skanera QR służy do szybkiego odczytywania kodów. Po zeskanowaniu kodu aplikacja może automatycznie dodać kupon lub wyświetlić powiązane informacje.



---



## 8. Zalety zastosowanego rozwiązania



Najważniejsze zalety aplikacji:



* szybki dostęp do kuponów,

* możliwość przypisywania kuponów do sklepów,

* wygodne skanowanie kodów QR,

* przejrzysty interfejs użytkownika,

* wykorzystanie wydajnego backendu w Rust,

* lekkość aplikacji dzięki Tauri,

* możliwość dalszego rozwoju o dodatkowe moduły,

* częściowa obsługa skanowania paragonów na komputerze.



---



## 9. Ograniczenia projektu



Projekt posiada również kilka ograniczeń:



* moduł skanowania paragonów działa tylko na komputerze,

* wersja mobilna może mieć ograniczony dostęp do niektórych funkcji systemowych,

* skuteczność skanowania QR zależy od jakości aparatu i oświetlenia,

* aplikacja wymaga dalszego dopracowania pod kątem obsługi błędów,

* konieczne może być rozszerzenie sposobu przechowywania danych.



---



## 10. Możliwości dalszego rozwoju



W przyszłości aplikację można rozbudować o dodatkowe funkcje, takie jak:



* synchronizacja danych między urządzeniami,

* automatyczne przypomnienia o kończących się kuponach,

* filtrowanie kuponów po kategorii,

* historia użytych kuponów,

* integracja z kontami użytkowników,

* automatyczne pobieranie kuponów z wybranych sklepów,

* rozwinięcie modułu skanowania paragonów również na urządzenia mobilne,

* analiza zakupów i sugerowanie kuponów na podstawie historii.



---



## 11. Podsumowanie



Aplikacja mobilna zbudowana przy użyciu **Tauri**, **Vue.js** oraz **Rust** stanowi praktyczne narzędzie do zarządzania kuponami promocyjnymi. Dzięki możliwości dodawania sklepów, przypisywania do nich kuponów oraz skanowania kodów QR użytkownik może bardzo szybko znaleźć potrzebną promocję.



Największą zaletą projektu jest prostota użytkowania oraz oszczędność czasu podczas zakupów. Dodatkowym elementem projektu jest moduł skanowania paragonów, który obecnie działa wyłącznie na komputerze, ale może być rozwinięty w przyszłości jako ważna funkcja wspierająca analizę zakupów.



Projekt pokazuje, że połączenie **Tauri**, **Vue.js** i **Rust** pozwala stworzyć lekką, wydajną i nowoczesną aplikację, która może działać jako wygodne narzędzie codziennego użytku.



