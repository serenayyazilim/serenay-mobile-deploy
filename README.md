# Serenay Mobile Deploy

Flutter tabanlı mobil uygulamaları App Store, Google Play ve AppGallery'e deploy etmek için masaüstü uygulaması. [Tauri](https://tauri.app) + [SvelteKit](https://kit.svelte.dev) ile geliştirilmiştir.

## Özellikler

- **Tek panelden çoklu platform deploy** — iOS (App Store Connect), Android (Google Play) ve Huawei (AppGallery) build/upload süreçlerini tek arayüzden yönetir.
- **Fastlane entegrasyonu** — Ruby tabanlı deploy script'i, projedeki `fastlane` metadata'sını (mağaza açıklamaları, locale'ler) okuyup kullanır.
- **App Store Connect yönetimi** — API key ile kimlik doğrulama, In-App Event oluşturma/düzenleme/gönderme, lokalizasyon ve ekran görüntüsü yükleme, ülke/bölge (territory) listeleme.
- **Sürüm senkronizasyonu** — `pubspec.yaml`, iOS ve Android proje dosyaları arasında versiyon/build numarasını tek komutla eşitler.
- **Çoklu proje / workspace desteği** — Tek bir workspace içinde birden fazla Flutter uygulamasını (`sermobileboss` modu) veya tekil projeleri (`generic` mod) otomatik algılar ve yönetir.
- **Firebase entegrasyonu** — Firebase hesap yönetimi ve proje oluşturma.
- **Sentry entegrasyonu** — Release/proje oluşturma ve kimlik doğrulama kontrolü (`~/.sentryclirc` veya ortam değişkeni üzerinden).
- **Slack bildirimleri** — Deploy başarılı/başarısız durumlarını webhook üzerinden Slack kanalına bildirir.
- **Mağaza yerelleştirmeleri** — Fastlane metadata dizinlerinden locale bazlı mağaza açıklamalarını çeker.

## Teknoloji

| Katman     | Teknoloji                          |
|------------|-------------------------------------|
| Arayüz     | SvelteKit 5, TypeScript, Tailwind CSS |
| Masaüstü   | Tauri 2 (Rust)                      |
| Deploy     | Ruby (Fastlane script'leri)         |

## Gereksinimler

- [Node.js](https://nodejs.org) 18+
- [Rust](https://www.rust-lang.org/tools/install) (stable) + Tauri sistem bağımlılıkları — bkz. [Tauri Prerequisites](https://tauri.app/start/prerequisites/)
- [Ruby](https://www.ruby-lang.org) (deploy script'leri için)
- Deploy edilecek projede kurulu ve yapılandırılmış [Fastlane](https://fastlane.tools)

## Kurulum

```bash
git clone https://github.com/serenayyazilim/serenay-mobile-deploy.git
cd serenay-mobile-deploy
npm install
```

### Geliştirme modunda çalıştırma

```bash
npm run tauri dev
```

### Production build

```bash
npm run tauri build
```

Derlenen uygulama `src-tauri/target/release/bundle/` altında oluşur.

## Yapılandırma

Aşağıdaki entegrasyonlar isteğe bağlıdır ve ortam değişkenleri ile açılır:

| Değişken              | Açıklama                                      |
|-----------------------|------------------------------------------------|
| `SLACK_WEBHOOK_URL`   | Deploy bildirimlerinin gönderileceği Slack webhook URL'i |
| `SENTRY_AUTH_TOKEN`   | Sentry API token'ı (alternatif olarak `~/.sentryclirc` okunur) |
| `SENTRY_ORG`          | Sentry organizasyon slug'ı                     |

App Store Connect kimlik bilgileri (Issuer ID, Key ID, `.p8` private key) uygulama içinden, her workspace'e özel olarak girilir ve yalnızca ilgili workspace dizininde saklanır — repoya veya başka bir yere gönderilmez.

## Proje yapısı

```
src/                    SvelteKit arayüz kodu
├─ lib/components/      UI bileşenleri
├─ lib/stores/          Svelte 5 runes tabanlı state
└─ routes/              Sayfa route'ları

src-tauri/               Rust (Tauri) backend
├─ src/appstoreconnect/  App Store Connect API istemcisi
├─ src/commands/         Frontend'e açılan Tauri komutları
├─ src/deploy/           Deploy süreç yönetimi
├─ src/firebase/         Firebase CLI entegrasyonu
├─ src/workspace/        Workspace algılama ve adaptörler
└─ scripts/              Fastlane tabanlı Ruby deploy script'leri
```

## Katkıda bulunma

Katkılar memnuniyetle karşılanır. Bir konu üzerinde çalışmaya başlamadan önce lütfen bir issue açarak neyi değiştirmek istediğinizi belirtin. Pull request göndermeden önce:

1. Repoyu fork'layın ve bir feature branch açın.
2. `npm run check` ile tip kontrolünü çalıştırın.
3. Değişikliklerinizi açıklayan net bir PR açıklaması yazın.

## Lisans

[MIT](LICENSE)
