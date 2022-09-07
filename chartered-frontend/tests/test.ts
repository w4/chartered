import { expect, test } from '@playwright/test';

test('redirect to login when unauthenticated', async ({ page }) => {
    await page.goto('/');
    expect(await page.textContent('h1')).toBe('chartered ✈️');
    expect(await page.textContent('button')).toBe('Login');
});

test('register button takes user to register page', async ({ page }) => {
    await page.goto('/');
    await page.locator('text=Register').click();
    await expect(page).toHaveURL(/.*register/);
    expect(await page.textContent('h1')).toBe('chartered ✈️');
    expect(await page.textContent('button')).toBe('Register');
});

test('can successfully register and login', async ({ page }) => {
    const username = Math.random().toString(36).substring(2,7);
    const password = 'aaaaaaaa';

    await page.goto('/');
    await page.locator('text=Register').click();

    await page.locator('input[id="username"]').fill(username);
    await page.locator('input[id="password"]').fill(password);
    await page.locator('text=Register').click();

    await expect(page.locator('[role="alert"]')).toHaveCount(0);

    await expect(page).toHaveURL(/.*login/);
    await page.locator('input[id="username"]').fill(username);
    await page.locator('input[id="password"]').fill(password);
    await page.locator('text=Username Password Login >> button').click();

    await expect(page).not.toHaveURL(/.*login/);
    expect(await page.textContent('h1')).toBe('Welcome to Chartered.');
});
