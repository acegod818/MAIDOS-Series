/**
 * Clean TypeScript example
 * This code should pass all Code-QC checks
 */

import { config } from './config.js';

// Constants are named meaningfully
const MAX_RETRY_COUNT = 3;
const DEFAULT_TIMEOUT_MS = 5000;
const API_VERSION = 'v1';

// Use environment variables for secrets
const apiKey = process.env.API_KEY;
const databaseUrl = process.env.DATABASE_URL;

/**
 * Fetches user data from the API
 * @param userId - The user's unique identifier
 * @returns The user object or null if not found
 */
export async function fetchUser(userId: string): Promise<User | null> {
  try {
    const response = await fetch(`${config.apiUrl}/users/${userId}`, {
      headers: {
        'Authorization': `Bearer ${apiKey}`,
        'Content-Type': 'application/json',
      },
    });

    if (!response.ok) {
      console.error(`Failed to fetch user ${userId}: ${response.status}`);
      return null;
    }

    return await response.json();
  } catch (error) {
    console.error('Network error fetching user:', error);
    throw new Error(`Failed to fetch user: ${error.message}`);
  }
}

/**
 * Validates user input
 * Uses early return to avoid deep nesting
 */
export function validateUserInput(input: UserInput): ValidationResult {
  if (!input.email) {
    return { valid: false, error: 'Email is required' };
  }

  if (!isValidEmail(input.email)) {
    return { valid: false, error: 'Invalid email format' };
  }

  if (!input.password) {
    return { valid: false, error: 'Password is required' };
  }

  if (input.password.length < 8) {
    return { valid: false, error: 'Password must be at least 8 characters' };
  }

  return { valid: true };
}

/**
 * Processes order with proper error handling
 */
export async function processOrder(orderData: OrderInput): Promise<OrderResult> {
  const { customerId, items, shippingAddress } = orderData;

  const customer = await fetchCustomer(customerId);
  const validatedItems = await validateOrderItems(items);
  const shippingCost = calculateShipping(shippingAddress);

  const order = await createOrder({
    customer,
    items: validatedItems,
    shippingCost,
  });

  await sendOrderConfirmation(customer.email, order);

  return {
    orderId: order.id,
    status: 'confirmed',
    estimatedDelivery: order.estimatedDelivery,
  };
}

// Helper functions are small and focused
function isValidEmail(email: string): boolean {
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  return emailRegex.test(email);
}

function calculateShipping(address: Address): number {
  const baseRate = 5.99;
  const distanceMultiplier = getDistanceMultiplier(address.zipCode);
  return baseRate * distanceMultiplier;
}

// Types are well-defined
interface User {
  id: string;
  email: string;
  name: string;
}

interface UserInput {
  email: string;
  password: string;
}

interface ValidationResult {
  valid: boolean;
  error?: string;
}

interface OrderInput {
  customerId: string;
  items: OrderItem[];
  shippingAddress: Address;
}

interface OrderResult {
  orderId: string;
  status: string;
  estimatedDelivery: Date;
}
