import http from 'k6/http';
import { check } from 'k6';
import { uuidv4 } from 'https://jslib.k6.io/k6-utils/1.4.0/index.js';

const TARGET_URL = __ENV.TARGET_URL || 'http://localhost:8080';
const TENANT_API_KEY = __ENV.TENANT_API_KEY || 'changeme-tenant-key';