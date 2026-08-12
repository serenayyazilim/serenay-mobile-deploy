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
  { id: 'basic', label: 'Basic Settings', icon: 'Settings' },
  { id: 'languages', label: 'Language Support', icon: 'Globe' },
  { id: 'registration', label: 'Registration Settings', icon: 'UserPlus' },
  { id: 'payment', label: 'Payment & Billing', icon: 'CreditCard' },
  { id: 'shipping', label: 'Shipping & Delivery', icon: 'Truck' },
  { id: 'products', label: 'Product Settings', icon: 'Package' },
  { id: 'features', label: 'Features', icon: 'Puzzle' },
];

export const SERCONF_FIELDS: SerConfField[] = [
  // ==================== BASIC SETTINGS ====================
  {
    key: 'API_URL',
    type: 'string',
    label: 'API URL',
    description: 'Enter the domain only (e.g. serb2b.com). https:// and /sermobileboss/ are added automatically.',
    category: 'basic',
    defaultValue: '',
    required: true,
  },
  {
    key: 'ILETISIM_URL',
    type: 'string',
    label: 'Contact URL',
    description: 'Enter the domain only (e.g. serb2b.com). https:// and /sermobileboss/ are added automatically.',
    category: 'basic',
    defaultValue: '',
  },
  {
    key: 'COMPANY_NAME',
    type: 'string',
    label: 'Company Name',
    description: 'Company name shown in the app',
    category: 'basic',
    defaultValue: '',
    required: true,
  },
  {
    key: 'COMPANY_IMAGE_LOGO',
    type: 'string',
    label: 'Logo Path',
    description: 'App logo file path (e.g. assets/images/logo.png)',
    category: 'basic',
    defaultValue: '',
  },
  {
    key: 'LOGIN_TYPE',
    type: 'enum',
    label: 'Login Type',
    description: 'User login method',
    category: 'basic',
    defaultValue: 'LoginType.mail',
    enumOptions: [
      { value: 'LoginType.mail', label: 'Email' },
      { value: 'LoginType.username', label: 'Username' },
      { value: 'LoginType.customerCode', label: 'Customer Code' },
    ],
  },
  {
    key: 'TEST_MAIL',
    type: 'string',
    label: 'Test Mail',
    description: 'Email/username used for test login',
    category: 'basic',
    defaultValue: 'mobile-test',
  },
  {
    key: 'DEFAULT_LANGUAGE',
    type: 'string',
    label: 'Default Language',
    description: 'Default language code (tr, en, ru, etc.)',
    category: 'basic',
    defaultValue: 'tr',
  },
  {
    key: 'DEFAULT_PHONE_CODE',
    type: 'string',
    label: 'Default Phone Code',
    description: 'Default country phone code (e.g. 90)',
    category: 'basic',
    defaultValue: '90',
  },
  {
    key: 'DEFAULT_COUNTRY',
    type: 'string',
    label: 'Default Country',
    description: 'Default country name',
    category: 'basic',
    defaultValue: 'Turkiye',
  },
  {
    key: 'SENTRY_DSN',
    type: 'string',
    label: 'Sentry DSN',
    description: 'Sentry DSN address for error tracking',
    category: 'basic',
    defaultValue: '',
  },

  // ==================== LANGUAGE SUPPORT ====================
  {
    key: 'ENGLISH',
    type: 'boolean',
    label: 'English',
    description: 'English language support',
    category: 'languages',
    defaultValue: true,
  },
  {
    key: 'RUSSIAN',
    type: 'boolean',
    label: 'Russian',
    description: 'Russian language support',
    category: 'languages',
    defaultValue: false,
  },
  {
    key: 'FRENCH',
    type: 'boolean',
    label: 'French',
    description: 'French language support',
    category: 'languages',
    defaultValue: false,
  },
  {
    key: 'ITALIAN',
    type: 'boolean',
    label: 'Italian',
    description: 'Italian language support',
    category: 'languages',
    defaultValue: false,
  },
  {
    key: 'ARABIC',
    type: 'boolean',
    label: 'Arabic',
    description: 'Arabic language support',
    category: 'languages',
    defaultValue: false,
  },
  {
    key: 'SPANISH',
    type: 'boolean',
    label: 'Spanish',
    description: 'Spanish language support',
    category: 'languages',
    defaultValue: false,
  },
  {
    key: 'KAZAKH',
    type: 'boolean',
    label: 'Kazakh',
    description: 'Kazakh language support',
    category: 'languages',
    defaultValue: false,
  },
  {
    key: 'KAZAKH_LANGUAGE_SUPPORT',
    type: 'boolean',
    label: 'Kazakh Extended Support',
    description: 'Additional language support features for Kazakh',
    category: 'languages',
    defaultValue: false,
  },
  {
    key: 'DEFAULT_LANGUAGE_CODE',
    type: 'number',
    label: 'Default Language Code',
    description: 'Default language ID number',
    category: 'languages',
    defaultValue: 213,
  },

  // ==================== REGISTRATION SETTINGS ====================
  {
    key: 'REGISTER_COMPANY_NAME',
    type: 'boolean',
    label: 'Company Name Field',
    description: 'Show company name field on the registration form',
    category: 'registration',
    defaultValue: true,
  },
  {
    key: 'REGISTER_CITY',
    type: 'boolean',
    label: 'City Field',
    description: 'Show city field on the registration form',
    category: 'registration',
    defaultValue: true,
  },
  {
    key: 'REGISTER_ADDRESS',
    type: 'boolean',
    label: 'Address Field',
    description: 'Show address field on the registration form',
    category: 'registration',
    defaultValue: true,
  },
  {
    key: 'ADDRESS_VISIBILITY',
    type: 'boolean',
    label: 'Address Visibility',
    description: 'Show the address section on the registration page',
    category: 'registration',
    defaultValue: true,
  },
  {
    key: 'MEMBER_TYPE',
    type: 'boolean',
    label: 'Member Type Option',
    description: 'Member type selection during registration (for Miniworld)',
    category: 'registration',
    defaultValue: false,
  },
  {
    key: 'PHONE_VISIBILITY_FUNCTION',
    type: 'boolean',
    label: 'Phone Required',
    description: 'Require phone number for the new iOS app',
    category: 'registration',
    defaultValue: false,
  },
  {
    key: 'CHANGE_PASSWORD_VISIBILITY',
    type: 'boolean',
    label: 'Change Password',
    description: 'Show the change password option',
    category: 'registration',
    defaultValue: false,
  },

  // ==================== PAYMENT & BILLING ====================
  {
    key: 'CUSTOMER_TYPE',
    type: 'boolean',
    label: 'Customer Type',
    description: 'Customer type selection on the payment page (individual/corporate)',
    category: 'payment',
    defaultValue: false,
  },
  {
    key: 'TC_NO',
    type: 'boolean',
    label: 'National ID No.',
    description: 'National ID number field',
    category: 'payment',
    defaultValue: false,
  },
  {
    key: 'VERGI_NO',
    type: 'boolean',
    label: 'Tax No.',
    description: 'Tax number field',
    category: 'payment',
    defaultValue: false,
  },
  {
    key: 'VERGI_DAIRESI',
    type: 'boolean',
    label: 'Tax Office',
    description: 'Tax office field',
    category: 'payment',
    defaultValue: false,
  },
  {
    key: 'PAYINFO',
    type: 'boolean',
    label: 'Payment Info',
    description: 'Show payment information',
    category: 'payment',
    defaultValue: false,
  },
  {
    key: 'KDV_PRICE',
    type: 'boolean',
    label: 'VAT Amount',
    description: 'Show VAT amount in the cart',
    category: 'payment',
    defaultValue: false,
  },
  {
    key: 'KDV_DISCOUNT',
    type: 'boolean',
    label: 'VAT Discount',
    description: 'VAT discount calculation',
    category: 'payment',
    defaultValue: false,
  },
  {
    key: 'GENERAL_AMOUNT_DESC',
    type: 'boolean',
    label: 'General Amount Description',
    description: 'Show the general amount description',
    category: 'payment',
    defaultValue: false,
  },

  // ==================== SHIPPING & DELIVERY ====================
  {
    key: 'CARGO_INFO',
    type: 'boolean',
    label: 'Shipping Info',
    description: 'Show shipping info in the "order now, pay later" option',
    category: 'shipping',
    defaultValue: true,
  },
  {
    key: 'SHIPPING_ADDRESS_VISIBILITY',
    type: 'boolean',
    label: 'Delivery Address',
    description: 'Address, details and contact person fields in payment options',
    category: 'shipping',
    defaultValue: true,
  },
  {
    key: 'GOODWAY_CARGO',
    type: 'boolean',
    label: 'Goodway Cargo',
    description: 'Goodway cargo integration',
    category: 'shipping',
    defaultValue: false,
  },

  // ==================== PRODUCT SETTINGS ====================
  {
    key: 'GRID_CARD_TUR',
    type: 'enum',
    label: 'Grid Card Type',
    description: 'Display type for product cards',
    category: 'products',
    defaultValue: 'YAZI',
    enumOptions: [
      { value: 'YAZI', label: 'Text' },
      { value: 'GORSEL', label: 'Image' },
    ],
  },
  {
    key: 'PRODUCT_IMAGE_ASPECT_RATIO',
    type: 'string',
    label: 'Image Aspect Ratio',
    description: 'Product image aspect ratio (e.g. 1200 / 1600)',
    category: 'products',
    defaultValue: '1200 / 1600',
  },
  {
    key: 'VARIANT_VISIBLE',
    type: 'boolean',
    label: 'Show Variants',
    description: 'Show product variants',
    category: 'products',
    defaultValue: true,
  },
  {
    key: 'CIRCLE_COLOR_VIEW',
    type: 'boolean',
    label: 'Round Color Swatches',
    description: 'Display product colors as round swatches',
    category: 'products',
    defaultValue: false,
  },
  {
    key: 'SUBTITLE2',
    type: 'boolean',
    label: 'Subtitle 2',
    description: 'Second subtitle field on the product card',
    category: 'products',
    defaultValue: false,
  },
  {
    key: 'PRIVATE_PRODUCT',
    type: 'boolean',
    label: 'Private Product',
    description: 'Show private product display',
    category: 'products',
    defaultValue: false,
  },
  {
    key: 'FIX_PRODUCT_IMAGE',
    type: 'boolean',
    label: 'Fixed Product Image',
    description: 'Display product image at a fixed size',
    category: 'products',
    defaultValue: false,
  },
  {
    key: 'GALLERY_SELECT_RADIUS',
    type: 'boolean',
    label: 'Gallery Selection Rounding',
    description: 'Rounded corners in gallery selection',
    category: 'products',
    defaultValue: false,
  },
  {
    key: 'GROUPLIST',
    type: 'boolean',
    label: 'Group List',
    description: 'Product add-ons grouped by category',
    category: 'products',
    defaultValue: false,
  },
  {
    key: 'MEASURE_LIST',
    type: 'boolean',
    label: 'Measurement List',
    description: 'Show product measurement list',
    category: 'products',
    defaultValue: false,
  },
  {
    key: 'CATEGORY_VISIBLE_CONTROL',
    type: 'boolean',
    label: 'Category Control',
    description: 'Redirect to categories when a category is tapped',
    category: 'products',
    defaultValue: false,
  },

  // ==================== FEATURES ====================
  {
    key: 'ALL_BRANDS',
    type: 'boolean',
    label: 'All Brands',
    description: 'Show the "all brands" button',
    category: 'features',
    defaultValue: true,
  },
  {
    key: 'BRANDS_API',
    type: 'boolean',
    label: 'Brands API',
    description: 'Use the API for the brand list',
    category: 'features',
    defaultValue: true,
  },
  {
    key: 'ADD_ALL_VARIANTS_BUTTON',
    type: 'boolean',
    label: 'Add All Variants Button',
    description: '"Add all colors to cart" button on product detail',
    category: 'features',
    defaultValue: false,
  },
  {
    key: 'ORDER_DETAIL',
    type: 'boolean',
    label: 'Order Detail',
    description: 'Order detail page feature',
    category: 'features',
    defaultValue: false,
  },
  {
    key: 'PHONE_NUMBER_NAME',
    type: 'boolean',
    label: 'Phone Number Labels',
    description: 'Label phone numbers on the contact page',
    category: 'features',
    defaultValue: false,
  },
  {
    key: 'SEE_DETAIL_BUTTON_VISIBILITY',
    type: 'boolean',
    label: 'See Detail Button',
    description: 'Show "see detail" button in the cart',
    category: 'features',
    defaultValue: true,
  },
  {
    key: 'IS_FAVORITE',
    type: 'boolean',
    label: 'Favorites',
    description: 'Add to favorites feature',
    category: 'features',
    defaultValue: true,
  },
  {
    key: 'WEB_VIEW',
    type: 'boolean',
    label: 'Web View',
    description: 'Web view feature',
    category: 'features',
    defaultValue: false,
  },
  {
    key: 'STUCK_VISIBLE',
    type: 'boolean',
    label: 'Stock Display',
    description: 'Show stock information',
    category: 'features',
    defaultValue: false,
  },
  {
    key: 'HANDLE_QTY',
    type: 'boolean',
    label: 'Quantity Control',
    description: 'Manual quantity entry',
    category: 'features',
    defaultValue: false,
  },
  {
    key: 'CUSTOM_SOUND',
    type: 'boolean',
    label: 'Custom Sound',
    description: 'Custom notification sound',
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
