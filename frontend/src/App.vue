<script setup lang="ts">
import { RouterLink, RouterView } from 'vue-router'
import { useTokenNavigation } from '@/composables/useTokenNavigation'

const { getRouteWithToken } = useTokenNavigation()
</script>

<template>
  <header>
    <div class="header-content">
      <div class="logo-section">
        <img alt="Vue logo" class="logo" src="@/assets/logo.svg" width="40" height="40" />
        <h1 class="app-title">Media Archive</h1>
      </div>

      <nav>
        <RouterLink :to="getRouteWithToken('/')">Home</RouterLink>
        <RouterLink :to="getRouteWithToken('/search')">Search</RouterLink>
        <RouterLink :to="getRouteWithToken('/status')">Status</RouterLink>
        <RouterLink :to="getRouteWithToken('/about')">About</RouterLink>
      </nav>
    </div>
  </header>

  <RouterView />
</template>

<style scoped>
header {
  background-color: var(--color-background-mute);
  border-bottom: 1px solid var(--color-border);
  padding: 1rem 0;
  position: sticky;
  top: 0;
  z-index: 100;
}

.header-content {
  max-width: 1200px;
  margin: 0 auto;
  padding: 0 2rem;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.logo-section {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.logo {
  display: block;
}

.app-title {
  font-size: 1.5rem;
  font-weight: 600;
  color: var(--color-heading);
  margin: 0;
}

nav {
  display: flex;
  gap: 2rem;
}

nav a {
  color: var(--color-text);
  text-decoration: none;
  padding: 0.5rem 1rem;
  border-radius: 6px;
  transition: all 0.3s;
  font-weight: 500;
}

nav a:hover {
  background-color: var(--color-background-soft);
  color: var(--color-text);
}

nav a.router-link-exact-active {
  background-color: var(--color-background-soft);
  color: var(--color-heading);
}

@media (max-width: 768px) {
  .header-content {
    padding: 0 1rem;
    flex-direction: column;
    gap: 1rem;
  }
  
  nav {
    gap: 1rem;
  }
  
  nav a {
    padding: 0.5rem 0.75rem;
    font-size: 0.9rem;
  }
}
</style>
