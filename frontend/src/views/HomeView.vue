<script setup lang="ts">
import { RouterLink } from 'vue-router'
import { useTokenNavigation } from '@/composables/useTokenNavigation'

const { getRouteWithToken } = useTokenNavigation()
</script>

<template>
  <main class="home">
    <div class="hero">
      <h1>Media Archive</h1>
      <p class="hero-description">
        Search and discover torrents with our powerful search engine
      </p>
      <RouterLink :to="getRouteWithToken('/search')" class="cta-button">
        Start Searching
      </RouterLink>
    </div>
    
    <div class="features">
      <div class="feature-card">
        <h3>🔍 Fast Search</h3>
        <p>Quickly find torrents using our optimized search API</p>
      </div>
      <div class="feature-card">
        <h3>📊 Detailed Info</h3>
        <p>View seeders, leechers, file sizes, and promotion details</p>
      </div>
      <div class="feature-card">
        <h3>📱 Responsive</h3>
        <p>Works perfectly on desktop, tablet, and mobile devices</p>
      </div>
    </div>
  </main>
</template>

<style scoped>
.home {
  max-width: 1200px;
  margin: 0 auto;
  padding: 2rem;
  text-align: center;
}

.hero {
  padding: 4rem 2rem;
  margin-bottom: 4rem;
}

.hero h1 {
  font-size: 3.5rem;
  color: var(--color-heading);
  margin-bottom: 1rem;
  font-weight: 700;
}

.hero-description {
  font-size: 1.25rem;
  color: var(--color-text-light);
  margin-bottom: 2rem;
  max-width: 600px;
  margin-left: auto;
  margin-right: auto;
}

.cta-button {
  display: inline-block;
  padding: 1rem 2rem;
  background: linear-gradient(135deg, var(--color-background-mute), var(--color-background-soft));
  color: var(--color-text);
  text-decoration: none;
  border-radius: 12px;
  font-size: 1.1rem;
  font-weight: 600;
  border: 2px solid var(--color-border);
  transition: all 0.3s;
}

.cta-button:hover {
  transform: translateY(-2px);
  border-color: var(--color-border-hover);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.features {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 2rem;
  margin-top: 2rem;
}

.feature-card {
  background: var(--color-background-mute);
  padding: 2rem;
  border-radius: 12px;
  border: 1px solid var(--color-border);
  transition: all 0.3s;
}

.feature-card:hover {
  transform: translateY(-4px);
  border-color: var(--color-border-hover);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.1);
}

.feature-card h3 {
  color: var(--color-heading);
  margin-bottom: 1rem;
  font-size: 1.4rem;
}

.feature-card p {
  color: var(--color-text-light);
  line-height: 1.6;
}

@media (max-width: 768px) {
  .home {
    padding: 1rem;
  }
  
  .hero {
    padding: 2rem 1rem;
  }
  
  .hero h1 {
    font-size: 2.5rem;
  }
  
  .features {
    grid-template-columns: 1fr;
  }
}
</style>
