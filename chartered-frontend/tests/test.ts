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
    const username = Math.random().toString(36).substring(2, 7);
    const password = 'aaaaaaaa';

    // navigate to register page
    await page.goto('/');
    await page.locator('text=Register').click();

    // register user
    await page.locator('input[id="username"]').fill(username);
    await page.locator('input[id="password"]').fill(password);
    await page.locator('text=Register').click();

    // ensure there were no errors
    await expect(page.locator('[role="alert"]')).toHaveCount(0);

    // login to user
    await expect(page).toHaveURL(/.*login/);
    await page.locator('input[id="username"]').fill(username);
    await page.locator('input[id="password"]').fill(password);
    await page.locator('text=Username Password Login >> button').click();

    // expect to be logged in
    await expect(page).not.toHaveURL(/.*login/);
    expect(await page.textContent('h1')).toBe('Welcome to Chartered.');
});

test('can create an organisation and add a user to it', async ({ page }) => {
    const username1 = Math.random().toString(36).substring(2, 7);
    const username2 = Math.random().toString(36).substring(2, 7);
    const password = 'aaaaaaaa';

    await page.goto('/');

    // create first user account
    await page.locator('text=Register').click();
    await page.locator('input[id="username"]').fill(username1);
    await page.locator('input[id="password"]').fill(password);
    await page.locator('text=Register').click();
    await expect(page).toHaveURL(/.*login/);

    // create second user account that we'll add to the organisation
    await page.locator('text=Register').click();
    await page.locator('input[id="username"]').fill(username2);
    await page.locator('input[id="password"]').fill(password);
    await page.locator('text=Register').click();
    await expect(page).toHaveURL(/.*login/);

    // login to first account
    await page.locator('input[id="username"]').fill(username1);
    await page.locator('input[id="password"]').fill(password);
    await page.locator('text=Username Password Login >> button').click();

    // navigate to create organisation page
    await page.locator('text=Organisations').click();
    await page.locator('text=+ Create').click();

    // create new organisation
    const organisation = Math.random().toString(36).substring(2, 7);
    await page.locator('input[id="name"]').fill(organisation);
    await page.locator('button:has-text("Create")').click();

    // open new organisation
    await page.locator(`text=${organisation}`).click();

    // navigate to members page
    await page.locator('main ul button:has-text("Members")').click();

    // add second user to organisation
    await page.locator('input[placeholder="Start typing a username..."]').fill(username2);
    await page.locator(`button:has-text("${username2}")`).click();
    await page
        .locator(`text=${username2} VISIBLE PUBLISH_VERSION YANK_VERSION MANAGE_USERS CREATE_CRATE Save >> button`)
        .click();

    // refresh the page to ensure the user was added
    await page.reload();

    // navigate to members page
    await page.locator('main ul button:has-text("Members")').click();

    // ensure both members are available
    await expect(page.locator(`text=${username1}`)).toHaveCount(1);
    await expect(page.locator(`text=${username2}`)).toHaveCount(1);
});
