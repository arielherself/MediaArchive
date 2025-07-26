import { useRoute } from 'vue-router'
import type { RouteLocationRaw } from 'vue-router'

export function useTokenNavigation() {
  const route = useRoute()
  
  const getRouteWithToken = (to: string | RouteLocationRaw): RouteLocationRaw => {
    const token = route.query.token as string
    
    if (typeof to === 'string') {
      return {
        path: to,
        query: token ? { token } : {}
      }
    }
    
    if (to.query && token) {
      return {
        ...to,
        query: { ...to.query, token }
      }
    }
    
    if (token) {
      return {
        ...to,
        query: { token }
      }
    }
    
    return to
  }
  
  return {
    getRouteWithToken
  }
} 