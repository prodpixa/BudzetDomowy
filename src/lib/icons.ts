import type { Component } from "svelte";
import {
  Baby,
  Bike,
  BookOpen,
  Bus,
  Car,
  Cat,
  CircleDashed,
  Coffee,
  CreditCard,
  Dog,
  Droplet,
  Dumbbell,
  Film,
  Fuel,
  Gamepad2,
  Gift,
  GraduationCap,
  HeartPulse,
  House,
  Music,
  Palette,
  PiggyBank,
  Pill,
  Plane,
  Repeat,
  Shield,
  Shirt,
  ShoppingCart,
  Smartphone,
  Sparkles,
  Trees,
  Tv,
  Utensils,
  Wifi,
  Wrench,
  Zap,
} from "@lucide/svelte";

/** Ikony dostępne dla kategorii (klucz zapisywany w bazie). */
export const CATEGORY_ICONS: Record<string, Component<any>> = {
  house: House,
  "shopping-cart": ShoppingCart,
  utensils: Utensils,
  car: Car,
  "heart-pulse": HeartPulse,
  repeat: Repeat,
  palette: Palette,
  coffee: Coffee,
  fuel: Fuel,
  bus: Bus,
  bike: Bike,
  plane: Plane,
  pill: Pill,
  zap: Zap,
  droplet: Droplet,
  wifi: Wifi,
  smartphone: Smartphone,
  tv: Tv,
  film: Film,
  music: Music,
  "gamepad-2": Gamepad2,
  "book-open": BookOpen,
  "graduation-cap": GraduationCap,
  dumbbell: Dumbbell,
  shirt: Shirt,
  gift: Gift,
  baby: Baby,
  dog: Dog,
  cat: Cat,
  trees: Trees,
  wrench: Wrench,
  sparkles: Sparkles,
  shield: Shield,
  "credit-card": CreditCard,
  "piggy-bank": PiggyBank,
};

export function categoryIcon(name: string | undefined): Component<any> {
  return (name && CATEGORY_ICONS[name]) || CircleDashed;
}

/** Slot palety → zmienna CSS; 0 = szary (bez kategorii). */
export function slotColor(slot: number): string {
  return `var(--s${slot >= 1 && slot <= 8 ? slot : 0})`;
}

export const SLOT_NAMES = [
  "Niebieski",
  "Pomarańczowy",
  "Morski",
  "Żółty",
  "Różowy",
  "Zielony",
  "Fioletowy",
  "Czerwony",
];
