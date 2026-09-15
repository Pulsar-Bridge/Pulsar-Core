import http from 'k6/http';
import { check } from 'k6';
import { uuidv4 } from 'https://jslib.k6.io/k6-utils/1.4.0/index.js';

const TARGET_URL = __ENV.TARGET_URL || 'http://localhost:8080';
const TENANT_API_KEY = __ENV.TENANT_API_KEY || 'changeme-tenant-key';

export const options = {
  scenarios: {
    webhook_burst: {
      executor: 'constant-arrival-rate',
      rate: 50,
      timeUnit: '1s',
      duration: '2m',
      preAllocatedVUs: 50,
      maxVUs: 200,
    },
  },
};

export default function () {
  const payload = JSON.stringify({
    external_deposit_id: uuidv4(),
    amount: '10.5000000',
    asset_code: 'USD',
    stellar_account: 'G' + 'A'.repeat(55),
  });

  const res = http.post(`${TARGET_URL}/webhook`, payload, {
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${TENANT_API_KEY}`,
      'X-Idempotency-Key': uuidv4(),
    },
  });

  check(res, {
    'status is 201 or 200': (r) => r.status === 201 || r.status === 200,
  });
}
