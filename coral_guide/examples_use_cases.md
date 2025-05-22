# Examples and Use Cases

This chapter provides illustrative examples of Coral programs, showcasing its unique blend of features and the new syntax. The examples aim to reflect Coral's goals: simple syntax, efficiency, modernity, cleanliness, and speed, with the Coral compiler and runtime handling significant underlying complexity to provide a smooth developer experience. All examples adhere to Coral's standard `(result, (error_id, error_description_string))` return tuple for functions and methods.

## 1. Concurrent Web Scraper/Data Processor

**Use Case:** Fetching titles from multiple web pages concurrently and storing them. This example highlights how Coral's actor model and persistence can simplify complex, I/O-bound, and stateful tasks.

**Coral Features Showcased:**

*   Actor model for concurrent tasks (an actor per URL).
*   Asynchronous method calls (`Future<(result, error_details)>`).
*   Persistent objects (`@persistent`) for storing scraped data and actor state.
*   String manipulation (conceptual).
*   Robust error handling using the `(result, error_details)` tuple.
*   `iter` loops for managing tasks.

```coral
// --- network_utils.cr (Conceptual Helper Module) ---
// This module simulates network operations for the example.
// In a real Coral application, this would use Coral's I/O and networking libraries.
def fetch_url_content(url_string):
    // Simulates fetching web content.
    if url_string eq "http://example.com/page1":
        html_content is "<html><head><title>Page 1 Title</title></head><body>Content</body></html>"
        return (html_content, (0, ""))
    elif url_string eq "http://example.com/page2":
        html_content is "<html><head><title>Another Title - Page 2</title></head><body>More Content</body></html>"
        return (html_content, (0, ""))
    elif url_string eq "http://example.com/error_page":
        return (null, (500, 'Simulated server error for {url_string}'))
    else:
        return (null, (404, 'URL not found: {url_string}'))
// --- End of network_utils.cr ---


// --- data_store.cr (Conceptual Data Definition Module) ---
@persistent // Marks ScrapedPage instances for automatic persistence
class ScrapedPage:
    url is ""
    title is ""
    processed_at is null // Could be a DateTime type provided by Coral's standard library

    def init(this, page_url_val, page_title_val):
        this.url is page_url_val
        this.title is page_title_val
        // (now_time, time_err) is Time.now() // Assume Time.now() returns (datetime, error_tuple)
        // if time_err.0 eq 0: this.processed_at is now_time
        this.processed_at is "timestamp_placeholder" // Simplified for example
        return (this, (0,"")) // Successful initialization
// --- End of data_store.cr ---


// --- scraper_actor.cr (Actor Definition Module) ---
import network_utils
import data_store

@persistent // Each scraper instance is an actor; its state is persistent.
class PageScraper:
    target_url is ""
    extracted_title is null
    status is "pending" // e.g., "pending", "success", "failed", "partial_success"
    error_message is ""

    def init(this, url_to_scrape_val):
        this.target_url is url_to_scrape_val
        return (this, (0,""))

    // Asynchronous method to perform the scraping.
    // When called on an actor reference, this returns Future<(String, (Int, String))>.
    def scrape(this):
        print('Starting scrape for: {this.target_url}')
        (content, fetch_err) is network_utils.fetch_url_content(this.target_url)

        if fetch_err.0 neq 0:
            this.status is "failed"
            this.error_message is fetch_err.1
            print('Failed to fetch {this.target_url}: {this.error_message}')
            return (null, fetch_err) // Propagate the fetch error

        // Simple title extraction (highly simplified for brevity).
        // A real application would use a robust HTML parsing library.
        TITLE_START_TAG is "<title>"
        TITLE_END_TAG is "</title>"
        // Assume string methods like 'find' and 'substring' also return (value, error_tuple)
        // and are handled correctly here (error checks omitted for this section's focus).
        start_index is content.find(TITLE_START_TAG).0 // .0 to get value, ignoring find's error tuple
        end_index is content.find(TITLE_END_TAG).0

        if start_index neq null and end_index neq null and start_index lt end_index:
            actual_start is start_index + TITLE_START_TAG.length
            // Assume content.substring also returns (value, error_tuple)
            this.extracted_title is content.substring(actual_start, end_index).0
            this.status is "success"
            print('Successfully scraped title from {this.target_url}: "{this.extracted_title}"')

            // Persist the successfully scraped data.
            // The Coral runtime handles the actual storage of the ScrapedPage object.
            (saved_data_obj, save_err) is data_store.ScrapedPage(this.target_url, this.extracted_title)
            if save_err.0 neq 0:
                print('Failed to save scraped data for {this.target_url}: {save_err.1}')
                this.status is "partial_success_save_failed"
                this.error_message is 'Save error: {save_err.1}'
                // Return successfully extracted title but indicate a non-critical save error.
                return (this.extracted_title, (201, this.error_message))
            
            return (this.extracted_title, (0, "")) // Success
        else:
            this.status is "failed"
            this.error_message is "Could not extract title from content."
            print('Failed to extract title from {this.target_url}')
            return (null, (101, this.error_message)) // Specific error for title extraction failure
// --- End of scraper_actor.cr ---


// --- main.cr (Main Orchestration Logic) ---
import scraper_actor // Import our actor definition

def main_scraper_orchestrator():
    URLS_TO_SCRAPE is [
        "http://example.com/page1",
        "http://example.com/page2",
        "http://example.com/error_page",
        "http://example.com/non_existent_page"
    ]

    scraper_futures is [] // To store futures returned by async actor calls

    iter URLS_TO_SCRAPE:
        url is it
        (scraper_instance, create_err) is scraper_actor.PageScraper(url)
        if create_err.0 eq 0:
            // Asynchronously call the 'scrape' method. This returns a Future immediately.
            // The Future will eventually resolve to the (String, (Int, String)) tuple from 'scrape'.
            future_scrape_result is scraper_instance.scrape()
            scraper_futures is scraper_futures + [{ "url": url, "future": future_scrape_result }]
        else:
            print('Error creating scraper actor for {url}: {create_err.1}')

    print('\n--- Scraping Results ---')
    // Conceptually, await the completion of all scrape operations.
    // Coral's scheduler would manage the execution of these asynchronous tasks.
    iter scraper_futures:
        task_info is it
        print('Waiting for result from: {task_info.url}')
        
        // Conceptual await: this "pauses" this iteration until the specific future resolves.
        (resolved_actor_tuple, future_system_err) is await task_info.future

        if future_system_err.0 eq 0: // Check if the Future itself resolved without system error
            (title, scrape_method_err) is resolved_actor_tuple // Unpack the (result, error) from scrape method
            if scrape_method_err.0 eq 0: // Success from scrape method
                print('SUCCESS for {task_info.url}: Title = "{title}"')
            elif scrape_method_err.0 eq 201: // Custom error: partial success (saved failed)
                print('PARTIAL SUCCESS for {task_info.url}: Title = "{title}", Note: {scrape_method_err.1}')
            else: // Other failure from scrape method
                print('FAILED for {task_info.url}: Error ID={scrape_method_err.0}, Desc="{scrape_method_err.1}"')
        else:
            // This means the future itself failed to resolve (e.g., actor crashed, system communication issue).
            print('SYSTEM ERROR resolving Future for {task_info.url}: {future_system_err.1}')
            
    return (true, (0,"")) // Overall orchestration considered successful

(run_main_result, main_run_err) is main_scraper_orchestrator()
if main_run_err.0 neq 0: print('Orchestrator error: {main_run_err.1}')
// --- End of main.cr ---
```

**Explanation:**
*   **Actors & Concurrency:** Each `PageScraper` is an actor. Multiple scrapers run concurrently, managed by Coral's runtime.
*   **Asynchronous Operations:** `scraper_instance.scrape()` returns a `Future` immediately. The main orchestrator collects these and can conceptually `await` their results, enabling efficient I/O.
*   **Persistence:** `@persistent` on `PageScraper` and `ScrapedPage` makes their instances' state durable. The Coral runtime handles saving this state automatically.
*   **Error Handling:** The `(result, error_details)` tuple is used consistently for function returns, actor method returns, and for the resolved values of Futures.
*   **Modularity:** The example is broken into conceptual modules.

## 2. Simple Persistent Counter Service

**Use Case:** A service maintaining a counter whose value persists across service restarts. This demonstrates the simplicity of creating stateful services with Coral's persistence.

**Coral Features Showcased:**
*   `@persistent` class (Actor) for the counter.
*   Automatic persistence of the counter's state by the Coral runtime.
*   Methods modifying state and returning new state or status.

```coral
// --- counter_service.cr ---
@persistent
class PersistentCounter:
    _count is 0 // Internal state, automatically persisted

    def init(this, initial_value):
        // (type_val, type_err) is type_of(initial_value) // Hypothetical runtime type check
        // if type_err.0 neq 0 or type_val neq "Integer":
        //     return (null, (1, "Initial value must be an integer"))
        // For simplicity, assume initial_value is correct type or init can't fail this way.
        this._count is initial_value
        print('PersistentCounter initialized with value: {this._count}')
        return (this, (0, ""))

    def increment(this, amount_val):
        // Assume amount_val is an Integer for simplicity. Robust code would check.
        if amount_val lt 0:
             return (this._count, (2, "Increment amount must be non-negative")) // Return current count on error
        this._count is this._count + amount_val
        print('Counter incremented by {amount_val}. New value: {this._count}')
        return (this._count, (0, "")) // Return the new count

    def get_current_value(this):
        print('Counter value requested: {this._count}')
        return (this._count, (0, ""))

    def reset(this):
        this._count is 0
        print('Counter reset to 0.')
        return (this._count, (0, ""))
// --- End of counter_service.cr ---


// --- main_counter_test.cr ---
import counter_service

def test_counter():
    INITIAL_COUNTER_VALUE is 10
    (my_counter, create_err) is counter_service.PersistentCounter(INITIAL_COUNTER_VALUE)

    if create_err.0 neq 0:
        print('Failed to create counter: {create_err.1}')
        return (false, create_err)

    // Actor method calls return Futures. We need to await them for results.
    // Conceptual async block for testing:
    async def interact_with_counter(counter_ref):
        (inc_res_tuple, fut_err1) is await counter_ref.increment(5)
        if fut_err1.0 eq 0: (new_val1, meth_err1) is inc_res_tuple; print('Incremented, new val: {new_val1}')
        else: print('Increment future/system error: {fut_err1.1}')

        (get_res_tuple, fut_err2) is await counter_ref.get_current_value()
        if fut_err2.0 eq 0: (cur_val2, meth_err2) is get_res_tuple; print('Current value: {cur_val2}')
        else: print('Get value future/system error: {fut_err2.1}')

        print("\nSimulating 're-accessing' the counter (state should be persistent)...")
        // In a real Coral system, if 'my_counter' was retrieved by a stable ID,
        // it would point to the same persistent actor with its state intact.
        // Here, we continue using the same reference for simplicity of example.
        (get_res_tuple2, fut_err3) is await counter_ref.get_current_value()
        if fut_err3.0 eq 0: (cur_val3, meth_err3) is get_res_tuple2; print('Value after re-access: {cur_val3}')
        else: print('Get value error after re-access: {fut_err3.1}')
        
        return (true, (0,""))

    print("Note: `await` calls below are conceptual for demonstrating Futures.")
    (run_interaction_res, interaction_err) is interact_with_counter(my_counter) // Conceptual direct call for example

    if interaction_err.0 neq 0:
        print("Interaction test failed: {interaction_err.1}")
    
    return (true, (0,""))

(test_run_res, test_run_err) is test_counter()
// --- End of main_counter_test.cr ---
```

**Explanation:**
*   **`@persistent` Actor:** `PersistentCounter` is an actor whose `_count` state is automatically managed by Coral's persistence.
*   **Automatic Persistence:** Changes to `this._count` are automatically persisted by the Coral runtime.
*   **Simplicity:** The developer focuses on the counter's logic; Coral handles concurrency and persistence.

## 3. Data Analysis and Reporting (Simplified)

**Use Case:** Processing a list of sales records to filter for high-value sales and then transform their data for a report. This demonstrates Coral's support for functional programming idioms for data manipulation.

**Coral Features Showcased:**
*   In-memory collections (lists).
*   Functional programming: `map_list`, `filter_list` (using tuple-based error handling).
*   Clarity of data transformation pipelines.
*   Custom classes for data records.

```coral
// --- data_analysis.cr ---

// Assume HOFs like 'map_list' and 'filter_list' are available,
// as defined in the Functional Programming chapter, handling (result, error) tuples.

class SalesRecord:
    product_id is ""
    region is ""
    amount is 0.0
    units is 0

    def init(this, pid_val, reg_val, amt_val, u_val):
        this.product_id is pid_val
        this.region is reg_val
        this.amount is amt_val
        this.units is u_val
        return (this, (0, "")) // Successful initialization

// Helper to create SalesRecord instances and handle errors, returning only the instance on success
// or null on error, simplifying list creation for the example.
def new_sales_record(pid, reg, amt, u):
    (record_instance, err) is SalesRecord(pid, reg, amt, u)
    if err.0 neq 0:
        print('Error creating SalesRecord ({pid}): {err.1}')
        return (null, err) // Propagate error
    return (record_instance, (0,""))


def process_sales_data(sales_records_list):
    def is_high_value_sale(record_obj):
        return (record_obj.amount gt 1000.0, (0, ""))

    def format_for_report(record_obj):
        revenue_per_unit_val is 0.0
        if record_obj.units neq 0:
            revenue_per_unit_val is record_obj.amount / record_obj.units
        else: // Avoid division by zero
            // Optionally, could return an error here:
            // return (null, (1, "Units cannot be zero for revenue calculation"))
            pass // Keep as 0.0 for this example if units are zero

        report_entry is { // Using Coral's map/dictionary literal syntax
            'item': record_obj.product_id,
            'zone': record_obj.region,
            'revenue': record_obj.amount,
            'items_sold': record_obj.units,
            'revenue_per_unit': revenue_per_unit_val
        }
        return (report_entry, (0, ""))

    (high_value_sales_list, filter_err) is filter_list(sales_records_list, is_high_value_sale)
    if filter_err.0 neq 0:
        print('Error during filtering: {filter_err.1}')
        return (null, filter_err)

    (report_data_list, map_err) is map_list(high_value_sales_list, format_for_report)
    if map_err.0 neq 0:
        print('Error during data transformation: {map_err.1}')
        return (null, map_err)

    return (report_data_list, (0, ""))


def generate_report():
    // Create a list of SalesRecord instances more robustly
    sales_data_tuples is [
        new_sales_record("ProdA", "North", 1200.0, 10),
        new_sales_record("ProdB", "South", 500.0, 50),
        new_sales_record("ProdC", "North", 2500.0, 5),
        new_sales_record("ProdD", "West", 800.0, 20),
        new_sales_record("ProdE", "South", 1500.0, 12),
        new_sales_record("ProdF", "North", 950.0, 30)
    ]

    RAW_SALES_DATA is []
    iter sales_data_tuples:
        (record_obj, err) is it
        if err.0 eq 0 and record_obj neq null:
            RAW_SALES_DATA is RAW_SALES_DATA + [record_obj]
        else:
            print('Skipping invalid record due to creation error: {err.1}')
    
    print("Valid Sales Records Count for Processing: {RAW_SALES_DATA.length}")

    (analysis_result_list, analysis_err) is process_sales_data(RAW_SALES_DATA)

    if analysis_err.0 eq 0:
        print("\n--- High-Value Sales Report ---")
        if analysis_result_list.length eq 0: // Assuming .length on lists
            print("No high-value sales found.")
        else:
            iter analysis_result_list:
                entry is it // 'entry' is a map/dictionary
                print('Product: {entry.item}, Region: {entry.zone}, Revenue: ${entry.revenue}, Units: {entry.items_sold}, Rev/Unit: ${entry.revenue_per_unit}')
    else:
        print("Failed to generate sales report: {analysis_err.1}")
        
    return (true, (0,""))

(report_run_res, report_run_err) is generate_report()
// --- End of data_analysis.cr ---
```

**Explanation:**
*   **Data Representation:** `SalesRecord` class holds sales information.
*   **Functional Processing:** `filter_list` and `map_list` demonstrate a clear, declarative data processing pipeline. The Coral compiler could potentially optimize such chains of HOF calls.
*   **Error Handling:** The HOFs and their helper functions use `(result, error_details)` tuples, ensuring robustness.
*   **Clarity:** The transformation steps are distinct and easy to follow.

These examples illustrate how Coral's features combine to create expressive, robust, and potentially concurrent and persistent applications, with the Coral system aiming to manage underlying complexities, providing a clean and modern development experience.
