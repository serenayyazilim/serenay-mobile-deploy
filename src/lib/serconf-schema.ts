export interface SerConfField {
  key: string;
  type: 'boolean' | 'string' | 'number' | 'enum';
  label: string;
  description: string;
  category: string;
  defaultValue: any;
  enumOptions?: { value: string; label: string }[];
  required?: boolean;
}

export interface SerConfCategory {
  id: string;
  label: string;
  icon: string;
}

export const SERCONF_CATEGORIES: SerConfCategory[] = [
  { id: 'basic', label: 'Temel Ayarlar', icon: 'Settings' },
  { id: 'languages', label: 'Dil Destegi', icon: 'Globe' },
  { id: 'registration', label: 'Kayit Ayarlari', icon: 'UserPlus' },
  { id: 'payment', label: 'Odeme & Fatura', icon: 'CreditCard' },
  { id: 'shipping', label: 'Kargo & Teslimat', icon: 'Truck' },
  { id: 'products', label: 'Urun Ayarlari', icon: 'Package' },
  { id: 'features', label: 'Ozellikler', icon: 'Puzzle' },
];

export const SERCONF_FIELDS: SerConfField[] = [
  // ==================== TEMEL AYARLAR ====================
  {
    key: 'API_URL',
    type: 'string',
    label: 'API URL',
    description: 'Sadece domain girin (ornek: serb2b.com). https:// ve /sermobileboss/ otomatik eklenir.',
    category: 'basic',
    defaultValue: '',
    required: true,
  },
  {
    key: 'ILETISIM_URL',
    type: 'string',
    label: 'Iletisim URL',
    description: 'Sadece domain girin (ornek: serb2b.com). https:// ve /sermobileboss/ otomatik eklenir.',
    category: 'basic',
    defaultValue: '',
  },
  {
    key: 'COMPANY_NAME',
    type: 'string',
    label: 'Sirket Adi',
    description: 'Uygulamada gorunecek sirket adi',
    category: 'basic',
    defaultValue: '',
    required: true,
  },
  {
    key: 'COMPANY_IMAGE_LOGO',
    type: 'string',
    label: 'Logo Path',
    description: 'Uygulama logosu dosya yolu (ornek: assets/images/logo.png)',
    category: 'basic',
    defaultValue: '',
  },
  {
    key: 'LOGIN_TYPE',
    type: 'enum',
    label: 'Giris Tipi',
    description: 'Kullanici giris yontemi',
    category: 'basic',
    defaultValue: 'LoginType.mail',
    enumOptions: [
      { value: 'LoginType.mail', label: 'E-posta' },
      { value: 'LoginType.username', label: 'Kullanici Adi' },
      { value: 'LoginType.customerCode', label: 'Musteri Kodu' },
    ],
  },
  {
    key: 'TEST_MAIL',
    type: 'string',
    label: 'Test Mail',
    description: 'Test girisinde kullanilacak mail/kullanici adi',
    category: 'basic',
    defaultValue: 'mobile-test',
  },
  {
    key: 'DEFAULT_LANGUAGE',
    type: 'string',
    label: 'Varsayilan Dil',
    description: 'Varsayilan dil kodu (tr, en, ru, vb.)',
    category: 'basic',
    defaultValue: 'tr',
  },
  {
    key: 'DEFAULT_PHONE_CODE',
    type: 'string',
    label: 'Varsayilan Telefon Kodu',
    description: 'Varsayilan ulke telefon kodu (ornek: 90)',
    category: 'basic',
    defaultValue: '90',
  },
  {
    key: 'DEFAULT_COUNTRY',
    type: 'string',
    label: 'Varsayilan Ulke',
    description: 'Varsayilan ulke adi',
    category: 'basic',
    defaultValue: 'Turkiye',
  },
  {
    key: 'SENTRY_DSN',
    type: 'string',
    label: 'Sentry DSN',
    description: 'Hata takibi icin Sentry DSN adresi',
    category: 'basic',
    defaultValue: '',
  },

  // ==================== DIL DESTEGI ====================
  {
    key: 'ENGLISH',
    type: 'boolean',
    label: 'Ingilizce',
    description: 'Ingilizce dil destegi',
    category: 'languages',
    defaultValue: true,
  },
  {
    key: 'RUSSIAN',
    type: 'boolean',
    label: 'Rusca',
    description: 'Rusca dil destegi',
    category: 'languages',
    defaultValue: false,
  },
  {
    key: 'FRENCH',
    type: 'boolean',
    label: 'Fransizca',
    description: 'Fransizca dil destegi',
    category: 'languages',
    defaultValue: false,
  },
  {
    key: 'ITALIAN',
    type: 'boolean',
    label: 'Italyanca',
    description: 'Italyanca dil destegi',
    category: 'languages',
    defaultValue: false,
  },
  {
    key: 'ARABIC',
    type: 'boolean',
    label: 'Arapca',
    description: 'Arapca dil destegi',
    category: 'languages',
    defaultValue: false,
  },
  {
    key: 'SPANISH',
    type: 'boolean',
    label: 'Ispanyolca',
    description: 'Ispanyolca dil destegi',
    category: 'languages',
    defaultValue: false,
  },
  {
    key: 'KAZAKH',
    type: 'boolean',
    label: 'Kazakca',
    description: 'Kazakca dil destegi',
    category: 'languages',
    defaultValue: false,
  },
  {
    key: 'KAZAKH_LANGUAGE_SUPPORT',
    type: 'boolean',
    label: 'Kazakca Ek Destek',
    description: 'Kazakca icin ek dil destegi ozellikleri',
    category: 'languages',
    defaultValue: false,
  },
  {
    key: 'DEFAULT_LANGUAGE_CODE',
    type: 'number',
    label: 'Varsayilan Dil Kodu',
    description: 'Varsayilan dil ID numarasi',
    category: 'languages',
    defaultValue: 213,
  },

  // ==================== KAYIT AYARLARI ====================
  {
    key: 'REGISTER_COMPANY_NAME',
    type: 'boolean',
    label: 'Sirket Adi Alani',
    description: 'Kayit formunda sirket adi alani gosterilsin mi',
    category: 'registration',
    defaultValue: true,
  },
  {
    key: 'REGISTER_CITY',
    type: 'boolean',
    label: 'Sehir Alani',
    description: 'Kayit formunda sehir alani gosterilsin mi',
    category: 'registration',
    defaultValue: true,
  },
  {
    key: 'REGISTER_ADDRESS',
    type: 'boolean',
    label: 'Adres Alani',
    description: 'Kayit formunda adres alani gosterilsin mi',
    category: 'registration',
    defaultValue: true,
  },
  {
    key: 'ADDRESS_VISIBILITY',
    type: 'boolean',
    label: 'Adres Gorunurlugu',
    description: 'Kayit sayfasinda adres bolumu gorunsun mu',
    category: 'registration',
    defaultValue: true,
  },
  {
    key: 'MEMBER_TYPE',
    type: 'boolean',
    label: 'Uye Tipi Secenegi',
    description: 'Kayit sirasinda uye tipi secimi (Miniworld icin)',
    category: 'registration',
    defaultValue: false,
  },
  {
    key: 'PHONE_VISIBILITY_FUNCTION',
    type: 'boolean',
    label: 'Telefon Zorunlu',
    description: 'iOS yeni uygulama icin telefon numarasi zorunlulugu',
    category: 'registration',
    defaultValue: false,
  },
  {
    key: 'CHANGE_PASSWORD_VISIBILITY',
    type: 'boolean',
    label: 'Sifre Degistirme',
    description: 'Sifre degistirme secenegi gorunsun mu',
    category: 'registration',
    defaultValue: false,
  },

  // ==================== ODEME & FATURA ====================
  {
    key: 'CUSTOMER_TYPE',
    type: 'boolean',
    label: 'Musteri Tipi',
    description: 'Odeme sayfasinda musteri tipi secimi (bireysel/kurumsal)',
    category: 'payment',
    defaultValue: false,
  },
  {
    key: 'TC_NO',
    type: 'boolean',
    label: 'TC No',
    description: 'TC Kimlik numarasi alani',
    category: 'payment',
    defaultValue: false,
  },
  {
    key: 'VERGI_NO',
    type: 'boolean',
    label: 'Vergi No',
    description: 'Vergi numarasi alani',
    category: 'payment',
    defaultValue: false,
  },
  {
    key: 'VERGI_DAIRESI',
    type: 'boolean',
    label: 'Vergi Dairesi',
    description: 'Vergi dairesi alani',
    category: 'payment',
    defaultValue: false,
  },
  {
    key: 'PAYINFO',
    type: 'boolean',
    label: 'Odeme Bilgisi',
    description: 'Odeme bilgisi gosterimi',
    category: 'payment',
    defaultValue: false,
  },
  {
    key: 'KDV_PRICE',
    type: 'boolean',
    label: 'KDV Tutari',
    description: 'Sepette KDV tutari gosterimi',
    category: 'payment',
    defaultValue: false,
  },
  {
    key: 'KDV_DISCOUNT',
    type: 'boolean',
    label: 'KDV Indirimi',
    description: 'KDV indirim hesaplamasi',
    category: 'payment',
    defaultValue: false,
  },
  {
    key: 'GENERAL_AMOUNT_DESC',
    type: 'boolean',
    label: 'Genel Tutar Aciklama',
    description: 'Genel tutar aciklamasi gosterimi',
    category: 'payment',
    defaultValue: false,
  },

  // ==================== KARGO & TESLIMAT ====================
  {
    key: 'CARGO_INFO',
    type: 'boolean',
    label: 'Kargo Bilgileri',
    description: 'Siparis ver sonra ode seceneginde kargo bilgileri',
    category: 'shipping',
    defaultValue: true,
  },
  {
    key: 'SHIPPING_ADDRESS_VISIBILITY',
    type: 'boolean',
    label: 'Teslimat Adresi',
    description: 'Odeme seceneklerinde adres, detay ve yetkili kisi alanlari',
    category: 'shipping',
    defaultValue: true,
  },
  {
    key: 'GOODWAY_CARGO',
    type: 'boolean',
    label: 'Goodway Kargo',
    description: 'Goodway kargo entegrasyonu',
    category: 'shipping',
    defaultValue: false,
  },

  // ==================== URUN AYARLARI ====================
  {
    key: 'GRID_CARD_TUR',
    type: 'enum',
    label: 'Grid Kart Tipi',
    description: 'Urun kartlarinin gorunum tipi',
    category: 'products',
    defaultValue: 'YAZI',
    enumOptions: [
      { value: 'YAZI', label: 'Yazili' },
      { value: 'GORSEL', label: 'Gorsel' },
    ],
  },
  {
    key: 'PRODUCT_IMAGE_ASPECT_RATIO',
    type: 'string',
    label: 'Resim Orani',
    description: 'Urun resmi en-boy orani (ornek: 1200 / 1600)',
    category: 'products',
    defaultValue: '1200 / 1600',
  },
  {
    key: 'VARIANT_VISIBLE',
    type: 'boolean',
    label: 'Varyant Gosterimi',
    description: 'Urun varyantlarini goster',
    category: 'products',
    defaultValue: true,
  },
  {
    key: 'CIRCLE_COLOR_VIEW',
    type: 'boolean',
    label: 'Yuvarlak Renk',
    description: 'Urun renklerini yuvarlak goruntule',
    category: 'products',
    defaultValue: false,
  },
  {
    key: 'SUBTITLE2',
    type: 'boolean',
    label: 'Alt Baslik 2',
    description: 'Urun kartinda ikinci alt baslik alani',
    category: 'products',
    defaultValue: false,
  },
  {
    key: 'PRIVATE_PRODUCT',
    type: 'boolean',
    label: 'Ozel Urun',
    description: 'Ozel urun gosterimi',
    category: 'products',
    defaultValue: false,
  },
  {
    key: 'FIX_PRODUCT_IMAGE',
    type: 'boolean',
    label: 'Sabit Urun Resmi',
    description: 'Urun resmini sabit boyutta goster',
    category: 'products',
    defaultValue: false,
  },
  {
    key: 'GALLERY_SELECT_RADIUS',
    type: 'boolean',
    label: 'Galeri Secim Yuvarlatma',
    description: 'Galeri seciminde yuvarlatilmis koseler',
    category: 'products',
    defaultValue: false,
  },
  {
    key: 'GROUPLIST',
    type: 'boolean',
    label: 'Grup Listesi',
    description: 'Gruba gore urun eklentisi',
    category: 'products',
    defaultValue: false,
  },
  {
    key: 'MEASURE_LIST',
    type: 'boolean',
    label: 'Olcu Listesi',
    description: 'Urun olcu listesi gosterimi',
    category: 'products',
    defaultValue: false,
  },
  {
    key: 'CATEGORY_VISIBLE_CONTROL',
    type: 'boolean',
    label: 'Kategori Kontrolu',
    description: 'Kategoriye tiklandiginda kategorilere yonlendir',
    category: 'products',
    defaultValue: false,
  },

  // ==================== OZELLIKLER ====================
  {
    key: 'ALL_BRANDS',
    type: 'boolean',
    label: 'Tum Markalar',
    description: 'Tum markalar butonunu goster',
    category: 'features',
    defaultValue: true,
  },
  {
    key: 'BRANDS_API',
    type: 'boolean',
    label: 'Marka API',
    description: 'Marka listesi icin API kullan',
    category: 'features',
    defaultValue: true,
  },
  {
    key: 'ADD_ALL_VARIANTS_BUTTON',
    type: 'boolean',
    label: 'Tum Varyantlar Butonu',
    description: 'Urun detayda tum renkleri sepete ekle butonu',
    category: 'features',
    defaultValue: false,
  },
  {
    key: 'ORDER_DETAIL',
    type: 'boolean',
    label: 'Siparis Detayi',
    description: 'Siparis detay sayfasi ozelligi',
    category: 'features',
    defaultValue: false,
  },
  {
    key: 'PHONE_NUMBER_NAME',
    type: 'boolean',
    label: 'Telefon Isimlendirme',
    description: 'Iletisim sayfasinda telefonlari isimlendir',
    category: 'features',
    defaultValue: false,
  },
  {
    key: 'SEE_DETAIL_BUTTON_VISIBILITY',
    type: 'boolean',
    label: 'Detay Gor Butonu',
    description: 'Sepette detay gor butonu gosterimi',
    category: 'features',
    defaultValue: true,
  },
  {
    key: 'IS_FAVORITE',
    type: 'boolean',
    label: 'Favoriler',
    description: 'Favorilere ekleme ozelligi',
    category: 'features',
    defaultValue: true,
  },
  {
    key: 'WEB_VIEW',
    type: 'boolean',
    label: 'Web Gorunumu',
    description: 'Web view ozelligi',
    category: 'features',
    defaultValue: false,
  },
  {
    key: 'STUCK_VISIBLE',
    type: 'boolean',
    label: 'Stok Gosterimi',
    description: 'Stok bilgisi gosterimi',
    category: 'features',
    defaultValue: false,
  },
  {
    key: 'HANDLE_QTY',
    type: 'boolean',
    label: 'Miktar Kontrolu',
    description: 'Manuel miktar girisi',
    category: 'features',
    defaultValue: false,
  },
  {
    key: 'CUSTOM_SOUND',
    type: 'boolean',
    label: 'Ozel Ses',
    description: 'Ozel bildirim sesi',
    category: 'features',
    defaultValue: false,
  },
];

// Helper: Get fields by category
export function getFieldsByCategory(categoryId: string): SerConfField[] {
  return SERCONF_FIELDS.filter(field => field.category === categoryId);
}

// Helper: Get required fields
export function getRequiredFields(): SerConfField[] {
  return SERCONF_FIELDS.filter(field => field.required);
}

// Helper: Get basic fields for create dialog (subset of most important settings)
export function getBasicFields(): SerConfField[] {
  const basicKeys = [
    'API_URL',
    'ILETISIM_URL',
    'COMPANY_NAME',
    'LOGIN_TYPE',
    'ENGLISH',
    'RUSSIAN',
  ];
  return SERCONF_FIELDS.filter(field => basicKeys.includes(field.key));
}
