package com.uiautomator.testapp;

import android.animation.AnimatorSet;
import android.animation.ObjectAnimator;
import android.os.Bundle;
import android.view.View;
import android.view.animation.AccelerateDecelerateInterpolator;
import android.widget.TextView;
import androidx.appcompat.app.AppCompatActivity;
import java.util.ArrayList;
import java.util.List;

public class AnimationsActivity extends AppCompatActivity {

    private View animationTarget;
    private TextView tvAnimationStatus;
    private List<AnimatorSet> runningAnimations = new ArrayList<>();

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_animations);

        animationTarget = findViewById(R.id.view_animation_target);
        tvAnimationStatus = findViewById(R.id.tv_animation_status);

        setupButtons();
    }

    private void setupButtons() {
        // Fade animation
        findViewById(R.id.btn_fade).setOnClickListener(v -> {
            tvAnimationStatus.setText("Status: Fade animation running");
            ObjectAnimator fadeOut = ObjectAnimator.ofFloat(animationTarget, "alpha", 1f, 0f);
            fadeOut.setDuration(1000);
            ObjectAnimator fadeIn = ObjectAnimator.ofFloat(animationTarget, "alpha", 0f, 1f);
            fadeIn.setDuration(1000);

            AnimatorSet animatorSet = new AnimatorSet();
            animatorSet.playSequentially(fadeOut, fadeIn);
            animatorSet.addListener(new android.animation.AnimatorListenerAdapter() {
                @Override
                public void onAnimationEnd(android.animation.Animator animation) {
                    tvAnimationStatus.setText("Status: Fade animation completed");
                }
            });
            animatorSet.start();
            runningAnimations.add(animatorSet);
        });

        // Slide animation
        findViewById(R.id.btn_slide).setOnClickListener(v -> {
            tvAnimationStatus.setText("Status: Slide animation running");
            ObjectAnimator slideRight = ObjectAnimator.ofFloat(animationTarget, "translationX", 0f, 300f);
            slideRight.setDuration(1000);
            ObjectAnimator slideBack = ObjectAnimator.ofFloat(animationTarget, "translationX", 300f, 0f);
            slideBack.setDuration(1000);

            AnimatorSet animatorSet = new AnimatorSet();
            animatorSet.playSequentially(slideRight, slideBack);
            animatorSet.addListener(new android.animation.AnimatorListenerAdapter() {
                @Override
                public void onAnimationEnd(android.animation.Animator animation) {
                    tvAnimationStatus.setText("Status: Slide animation completed");
                }
            });
            animatorSet.start();
            runningAnimations.add(animatorSet);
        });

        // Rotate animation
        findViewById(R.id.btn_rotate).setOnClickListener(v -> {
            tvAnimationStatus.setText("Status: Rotate animation running");
            ObjectAnimator rotate = ObjectAnimator.ofFloat(animationTarget, "rotation", 0f, 360f);
            rotate.setDuration(2000);
            rotate.setInterpolator(new AccelerateDecelerateInterpolator());
            rotate.addListener(new android.animation.AnimatorListenerAdapter() {
                @Override
                public void onAnimationEnd(android.animation.Animator animation) {
                    tvAnimationStatus.setText("Status: Rotate animation completed");
                }
            });
            rotate.start();
        });

        // Scale animation
        findViewById(R.id.btn_scale).setOnClickListener(v -> {
            tvAnimationStatus.setText("Status: Scale animation running");
            ObjectAnimator scaleX = ObjectAnimator.ofFloat(animationTarget, "scaleX", 1f, 2f, 1f);
            ObjectAnimator scaleY = ObjectAnimator.ofFloat(animationTarget, "scaleY", 1f, 2f, 1f);

            AnimatorSet animatorSet = new AnimatorSet();
            animatorSet.playTogether(scaleX, scaleY);
            animatorSet.setDuration(2000);
            animatorSet.addListener(new android.animation.AnimatorListenerAdapter() {
                @Override
                public void onAnimationEnd(android.animation.Animator animation) {
                    tvAnimationStatus.setText("Status: Scale animation completed");
                }
            });
            animatorSet.start();
            runningAnimations.add(animatorSet);
        });

        // Stop all animations
        findViewById(R.id.btn_stop_all).setOnClickListener(v -> {
            for (AnimatorSet animatorSet : runningAnimations) {
                if (animatorSet.isRunning()) {
                    animatorSet.cancel();
                }
            }
            runningAnimations.clear();
            animationTarget.setAlpha(1f);
            animationTarget.setTranslationX(0f);
            animationTarget.setTranslationY(0f);
            animationTarget.setRotation(0f);
            animationTarget.setScaleX(1f);
            animationTarget.setScaleY(1f);
            tvAnimationStatus.setText("Status: All animations stopped");
        });
    }
}
