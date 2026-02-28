package com.uiautomator.testapp;

import android.os.Bundle;
import android.view.View;
import android.view.ViewGroup;
import android.widget.ArrayAdapter;
import android.widget.ListView;
import android.widget.ScrollView;
import android.widget.TextView;
import androidx.annotation.NonNull;
import androidx.appcompat.app.AppCompatActivity;
import androidx.recyclerview.widget.LinearLayoutManager;
import androidx.recyclerview.widget.RecyclerView;
import com.google.android.material.tabs.TabLayout;
import java.util.ArrayList;
import java.util.List;

public class ListsActivity extends AppCompatActivity {

    private TabLayout tabLayout;
    private ListView listView;
    private RecyclerView recyclerView;
    private ScrollView scrollView;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_lists);

        tabLayout = findViewById(R.id.tab_layout);
        listView = findViewById(R.id.list_view);
        recyclerView = findViewById(R.id.recycler_view);
        scrollView = findViewById(R.id.scroll_view);

        setupTabs();
        setupListView();
        setupRecyclerView();
        setupScrollView();

        // Show ListView by default
        showListView();
    }

    private void setupTabs() {
        tabLayout.addTab(tabLayout.newTab().setText("ListView"));
        tabLayout.addTab(tabLayout.newTab().setText("RecyclerView"));
        tabLayout.addTab(tabLayout.newTab().setText("ScrollView"));

        tabLayout.addOnTabSelectedListener(new TabLayout.OnTabSelectedListener() {
            @Override
            public void onTabSelected(TabLayout.Tab tab) {
                switch (tab.getPosition()) {
                    case 0: showListView(); break;
                    case 1: showRecyclerView(); break;
                    case 2: showScrollView(); break;
                }
            }

            @Override
            public void onTabUnselected(TabLayout.Tab tab) {}

            @Override
            public void onTabReselected(TabLayout.Tab tab) {}
        });
    }

    private void setupListView() {
        List<String> items = new ArrayList<>();
        for (int i = 1; i <= 100; i++) {
            items.add("ListView Item " + i);
        }
        ArrayAdapter<String> adapter = new ArrayAdapter<>(this,
            android.R.layout.simple_list_item_1, items);
        listView.setAdapter(adapter);
    }

    private void setupRecyclerView() {
        List<String> items = new ArrayList<>();
        for (int i = 1; i <= 1000; i++) {
            items.add("RecyclerView Item " + i);
        }
        recyclerView.setLayoutManager(new LinearLayoutManager(this));
        recyclerView.setAdapter(new RecyclerAdapter(items));
    }

    private void setupScrollView() {
        TextView tvContent = findViewById(R.id.tv_scroll_content);
        StringBuilder content = new StringBuilder();
        for (int i = 1; i <= 100; i++) {
            content.append("Paragraph ").append(i).append("\n");
            content.append("This is a long text content for testing scrolling. ");
            content.append("Lorem ipsum dolor sit amet, consectetur adipiscing elit. ");
            content.append("Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.\n\n");
        }
        tvContent.setText(content.toString());
    }

    private void showListView() {
        listView.setVisibility(View.VISIBLE);
        recyclerView.setVisibility(View.GONE);
        scrollView.setVisibility(View.GONE);
    }

    private void showRecyclerView() {
        listView.setVisibility(View.GONE);
        recyclerView.setVisibility(View.VISIBLE);
        scrollView.setVisibility(View.GONE);
    }

    private void showScrollView() {
        listView.setVisibility(View.GONE);
        recyclerView.setVisibility(View.GONE);
        scrollView.setVisibility(View.VISIBLE);
    }

    // RecyclerView Adapter
    private static class RecyclerAdapter extends RecyclerView.Adapter<RecyclerAdapter.ViewHolder> {
        private final List<String> items;

        RecyclerAdapter(List<String> items) {
            this.items = items;
        }

        @NonNull
        @Override
        public ViewHolder onCreateViewHolder(@NonNull ViewGroup parent, int viewType) {
            TextView textView = new TextView(parent.getContext());
            textView.setLayoutParams(new ViewGroup.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT,
                ViewGroup.LayoutParams.WRAP_CONTENT));
            textView.setPadding(32, 32, 32, 32);
            textView.setTextSize(16);
            return new ViewHolder(textView);
        }

        @Override
        public void onBindViewHolder(@NonNull ViewHolder holder, int position) {
            holder.textView.setText(items.get(position));
        }

        @Override
        public int getItemCount() {
            return items.size();
        }

        static class ViewHolder extends RecyclerView.ViewHolder {
            TextView textView;

            ViewHolder(TextView textView) {
                super(textView);
                this.textView = textView;
            }
        }
    }
}
