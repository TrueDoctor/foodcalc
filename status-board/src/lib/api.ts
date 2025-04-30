
// For production deployment
const API_BASE_URL = 'https://essen.campus-kit.de/api';
// For local development
// const API_BASE_URL = 'http://localhost:8090';

interface MealStatus {
    meal_id: number;
    [key: string]: any;
}

interface Meal {
    status: {
        end: number;
        [key: string]: any;
    };
    [key: string]: any;
}

interface Event {
    id: number;
    name: string;
    [key: string]: any;
}

/**
 * Fetches meals for a specific event or the next upcoming event
 * @param {number|null} eventId - Optional event ID to fetch meals for
 * @returns {Promise<Array>} - Array of meal data grouped by day
 */
export async function fetchMeals(eventId: number | null = null): Promise<any[]> {
    let url = API_BASE_URL;
    
    if (eventId) {
        url += `?event=${eventId}`;
    }
    
    try {
        const response = await fetch(url);
        
        if (!response.ok) {
            if (response.status === 404) {
                throw new Error(`No meals found for this event`);
            }
            throw new Error(`Failed to fetch meals: ${response.status}`);
        }
        
        return await response.json();
    } catch (error) {
        console.error('Error fetching meals:', error);
        throw error;
    }
}

/**
 * Fetches a list of all available events
 * @returns {Promise<Array>} - Array of event objects with id and name
 */
export async function fetchEvents(): Promise<Event[]> {
    const response = await fetch(`${API_BASE_URL}/events`);
    
    if (!response.ok) {
        throw new Error(`Failed to fetch events: ${response.status}`);
    }
    
    return response.json();
}

/**
 * Fetches details for a specific event
 * @param {number} eventId - The ID of the event to fetch
 * @returns {Promise<Object>} - Event details object
 */
export async function fetchEventDetails(eventId: number): Promise<any> {
    const response = await fetch(`${API_BASE_URL}/events/${eventId}`);
    
    if (!response.ok) {
        throw new Error(`Failed to fetch event details: ${response.status}`);
    }
    
    return response.json();
}

/**
 * Updates the status of a meal
 * @param {Object} mealStatus - The updated meal status object
 * @returns {Promise<Array>} - Array of updated meal statuses
 */
export async function updateMealStatus(mealStatus: MealStatus): Promise<any[]> {
    try {
        // Create a clean copy of the status object to prevent reference issues
        const statusCopy: MealStatus = { ...mealStatus };
        
        console.log("Updating meal status:", statusCopy);
        
        const response = await fetch(`${API_BASE_URL}/${statusCopy.meal_id}`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(statusCopy),
        });
        
        if (!response.ok) {
            throw new Error(`Failed to update meal status: ${response.status}`);
        }
        
        return await response.json();
    } catch (error) {
        console.error('Update meal status error:', error);
        throw error;
    }
}

/**
 * Helper function to check if a meal is currently active (not in the past)
 * @param {Object} meal - The meal object to check
 * @returns {boolean} - Whether the meal is active
 */
export function isMealActive(meal: Meal): boolean {
    const now = Math.floor(Date.now() / 1000);
    // Consider a meal active if it's ending time is in the future
    return meal.status.end > now;
}

/**
 * Format a Unix timestamp to a locale date string
 * @param {number} timestamp - Unix timestamp in seconds
 * @param {Object} options - Intl.DateTimeFormat options
 * @returns {string} - Formatted date string
 */
export function formatDate(timestamp: number, options: Intl.DateTimeFormatOptions = {}): string {
    // Force UTC timezone to prevent browser timezone adjustments
    const date = new Date(timestamp * 1000);
    return date.toLocaleDateString('de-DE', {
        ...options,
        timeZone: 'UTC'
    });
}

/**
 * Format a Unix timestamp to a locale time string
 * @param {number} timestamp - Unix timestamp in seconds
 * @param {Object} options - Intl.DateTimeFormat options
 * @returns {string} - Formatted time string
 */
export function formatTime(timestamp: number, options: Intl.DateTimeFormatOptions = {}): string {
    // Force UTC timezone to prevent browser timezone adjustments
    const date = new Date(timestamp * 1000);
    return date.toLocaleTimeString('de-DE', {
        hour: 'numeric',
        minute: 'numeric',
        ...options,
        timeZone: 'UTC'
    });
}

/**
 * Get the day key for a meal to properly group by days
 * @param {number} timestamp - Unix timestamp in seconds
 * @returns {string} - YYYY-MM-DD format date string for grouping
 */
export function getDayKey(timestamp: number): string {
    // Adjust for the -3 hour offset in the server code
    // The server uses: OffsetDateTime::from_unix_timestamp(time - 3 * hour)
    const adjustedTimestamp = timestamp - (3 * 3600);
    const date = new Date(adjustedTimestamp * 1000);
    
    const year = date.getUTCFullYear();
    const month = String(date.getUTCMonth() + 1).padStart(2, '0');
    const day = String(date.getUTCDate()).padStart(2, '0');
    return `${year}-${month}-${day}`;
}
